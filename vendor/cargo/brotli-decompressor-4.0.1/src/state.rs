#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]


use alloc;
use core;
use context::kContextLookup;
use bit_reader::{BrotliBitReader, BrotliGetAvailableBits, BrotliInitBitReader};
use huffman::{BROTLI_HUFFMAN_MAX_CODE_LENGTH, BROTLI_HUFFMAN_MAX_CODE_LENGTHS_SIZE,
              BROTLI_HUFFMAN_MAX_TABLE_SIZE, HuffmanCode, HuffmanTreeGroup};
use alloc::SliceWrapper;

#[allow(dead_code)]
pub enum WhichTreeGroup {
  LITERAL,
  INSERT_COPY,
  DISTANCE,
}
#[repr(C)]
#[derive(Clone,Copy, Debug)]
pub enum BrotliDecoderErrorCode{
  BROTLI_DECODER_NO_ERROR = 0,
  /* Same as BrotliDecoderResult values */
  BROTLI_DECODER_SUCCESS = 1,
  BROTLI_DECODER_NEEDS_MORE_INPUT = 2,
  BROTLI_DECODER_NEEDS_MORE_OUTPUT = 3,

  /* Errors caused by invalid input */
  BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_NIBBLE = -1,
  BROTLI_DECODER_ERROR_FORMAT_RESERVED = -2,
  BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_META_NIBBLE = -3,
  BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_ALPHABET = -4,
  BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_SAME = -5,
  BROTLI_DECODER_ERROR_FORMAT_CL_SPACE = -6,
  BROTLI_DECODER_ERROR_FORMAT_HUFFMAN_SPACE = -7,
  BROTLI_DECODER_ERROR_FORMAT_CONTEXT_MAP_REPEAT = -8,
  BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_1 = -9,
  BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_2 = -10,
  BROTLI_DECODER_ERROR_FORMAT_TRANSFORM = -11,
  BROTLI_DECODER_ERROR_FORMAT_DICTIONARY = -12,
  BROTLI_DECODER_ERROR_FORMAT_WINDOW_BITS = -13,
  BROTLI_DECODER_ERROR_FORMAT_PADDING_1 = -14,
  BROTLI_DECODER_ERROR_FORMAT_PADDING_2 = -15,
  BROTLI_DECODER_ERROR_FORMAT_DISTANCE = -16,

  /* -17..-18 codes are reserved */

  BROTLI_DECODER_ERROR_DICTIONARY_NOT_SET = -19,
  BROTLI_DECODER_ERROR_INVALID_ARGUMENTS = -20,

  /* Memory allocation problems */
  BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MODES = -21,
  /* Literal = insert and distance trees together */
  BROTLI_DECODER_ERROR_ALLOC_TREE_GROUPS = -22,
  /* -23..-24 codes are reserved for distinct tree groups */
  BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MAP = -25,
  BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_1 = -26,
  BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_2 = -27,
  /* -28..-29 codes are reserved for dynamic ring-buffer allocation */
  BROTLI_DECODER_ERROR_ALLOC_BLOCK_TYPE_TREES = -30,

  /* "Impossible" states */
  BROTLI_DECODER_ERROR_UNREACHABLE = -31,
}

#[derive(Debug)]
pub enum BrotliRunningState {
  BROTLI_STATE_UNINITED,
  BROTLI_STATE_LARGE_WINDOW_BITS,
  BROTLI_STATE_INITIALIZE,
  BROTLI_STATE_METABLOCK_BEGIN,
  BROTLI_STATE_METABLOCK_HEADER,
  BROTLI_STATE_METABLOCK_HEADER_2,
  BROTLI_STATE_CONTEXT_MODES,
  BROTLI_STATE_COMMAND_BEGIN,
  BROTLI_STATE_COMMAND_INNER,
  BROTLI_STATE_COMMAND_POST_DECODE_LITERALS,
  BROTLI_STATE_COMMAND_POST_WRAP_COPY,
  BROTLI_STATE_UNCOMPRESSED,
  BROTLI_STATE_METADATA,
  BROTLI_STATE_COMMAND_INNER_WRITE,
  BROTLI_STATE_METABLOCK_DONE,
  BROTLI_STATE_COMMAND_POST_WRITE_1,
  BROTLI_STATE_COMMAND_POST_WRITE_2,
  BROTLI_STATE_HUFFMAN_CODE_0,
  BROTLI_STATE_HUFFMAN_CODE_1,
  BROTLI_STATE_HUFFMAN_CODE_2,
  BROTLI_STATE_HUFFMAN_CODE_3,
  BROTLI_STATE_CONTEXT_MAP_1,
  BROTLI_STATE_CONTEXT_MAP_2,
  BROTLI_STATE_TREE_GROUP,
  BROTLI_STATE_DONE,
}

pub enum BrotliRunningMetablockHeaderState {
  BROTLI_STATE_METABLOCK_HEADER_NONE,
  BROTLI_STATE_METABLOCK_HEADER_EMPTY,
  BROTLI_STATE_METABLOCK_HEADER_NIBBLES,
  BROTLI_STATE_METABLOCK_HEADER_SIZE,
  BROTLI_STATE_METABLOCK_HEADER_UNCOMPRESSED,
  BROTLI_STATE_METABLOCK_HEADER_RESERVED,
  BROTLI_STATE_METABLOCK_HEADER_BYTES,
  BROTLI_STATE_METABLOCK_HEADER_METADATA,
}
pub enum BrotliRunningUncompressedState {
  BROTLI_STATE_UNCOMPRESSED_NONE,
  BROTLI_STATE_UNCOMPRESSED_WRITE,
}

pub enum BrotliRunningTreeGroupState {
  BROTLI_STATE_TREE_GROUP_NONE,
  BROTLI_STATE_TREE_GROUP_LOOP,
}

pub enum BrotliRunningContextMapState {
  BROTLI_STATE_CONTEXT_MAP_NONE,
  BROTLI_STATE_CONTEXT_MAP_READ_PREFIX,
  BROTLI_STATE_CONTEXT_MAP_HUFFMAN,
  BROTLI_STATE_CONTEXT_MAP_DECODE,
  BROTLI_STATE_CONTEXT_MAP_TRANSFORM,
}

pub enum BrotliRunningHuffmanState {
  BROTLI_STATE_HUFFMAN_NONE,
  BROTLI_STATE_HUFFMAN_SIMPLE_SIZE,
  BROTLI_STATE_HUFFMAN_SIMPLE_READ,
  BROTLI_STATE_HUFFMAN_SIMPLE_BUILD,
  BROTLI_STATE_HUFFMAN_COMPLEX,
  BROTLI_STATE_HUFFMAN_LENGTH_SYMBOLS,
}

pub enum BrotliRunningDecodeUint8State {
  BROTLI_STATE_DECODE_UINT8_NONE,
  BROTLI_STATE_DECODE_UINT8_SHORT,
  BROTLI_STATE_DECODE_UINT8_LONG,
}

pub enum BrotliRunningReadBlockLengthState {
  BROTLI_STATE_READ_BLOCK_LENGTH_NONE,
  BROTLI_STATE_READ_BLOCK_LENGTH_SUFFIX,
}

pub const kLiteralContextBits: usize = 6;

pub struct BlockTypeAndLengthState<AllocHC: alloc::Allocator<HuffmanCode>> {
  pub substate_read_block_length: BrotliRunningReadBlockLengthState,
  pub num_block_types: [u32; 3],
  pub block_length_index: u32,
  pub block_length: [u32; 3],
  pub block_type_trees: AllocHC::AllocatedMemory,
  pub block_len_trees: AllocHC::AllocatedMemory,
  pub block_type_rb: [u32; 6],
}

pub struct BrotliState<AllocU8: alloc::Allocator<u8>,
                       AllocU32: alloc::Allocator<u32>,
                       AllocHC: alloc::Allocator<HuffmanCode>>
{
  pub state: BrotliRunningState,

  // This counter is reused for several disjoint loops.
  pub loop_counter: i32,
  pub br: BrotliBitReader,
  pub alloc_u8: AllocU8,
  pub alloc_u32: AllocU32,
  pub alloc_hc: AllocHC,
  // void* memory_manager_opaque,
  pub buffer: [u8; 8],
  pub buffer_length: u32,
  pub pos: i32,
  pub max_backward_distance: i32,
  pub max_backward_distance_minus_custom_dict_size: i32,
  pub max_distance: i32,
  pub ringbuffer_size: i32,
  pub ringbuffer_mask: i32,
  pub dist_rb_idx: i32,
  pub dist_rb: [i32; 4],
  pub ringbuffer: AllocU8::AllocatedMemory,
  // pub ringbuffer_end : usize,
  pub htree_command_index: u16,
  pub context_lookup: &'static [u8;512],
  pub context_map_slice_index: usize,
  pub dist_context_map_slice_index: usize,

  pub sub_loop_counter: u32,

  // This ring buffer holds a few past copy distances that will be used by */
  // some special distance codes.
  pub literal_hgroup: HuffmanTreeGroup<AllocU32, AllocHC>,
  pub insert_copy_hgroup: HuffmanTreeGroup<AllocU32, AllocHC>,
  pub distance_hgroup: HuffmanTreeGroup<AllocU32, AllocHC>,
  // This is true if the literal context map histogram type always matches the
  // block type. It is then not needed to keep the context (faster decoding).
  pub trivial_literal_context: i32,
  // Distance context is actual after command is decoded and before distance
  // is computed. After distance computation it is used as a temporary variable
  pub distance_context: i32,
  pub meta_block_remaining_len: i32,
  pub block_type_length_state: BlockTypeAndLengthState<AllocHC>,
  pub distance_postfix_bits: u32,
  pub num_direct_distance_codes: u32,
  pub distance_postfix_mask: i32,
  pub num_dist_htrees: u32,
  pub dist_context_map: AllocU8::AllocatedMemory,
  // NOT NEEDED? the index below seems to supersede it pub literal_htree : AllocHC::AllocatedMemory,
  pub literal_htree_index: u8,
  pub dist_htree_index: u8,
  pub large_window: bool,
  pub should_wrap_ringbuffer: bool,
  pub error_code: BrotliDecoderErrorCode,
  pub repeat_code_len: u32,
  pub prev_code_len: u32,

  pub copy_length: i32,
  pub distance_code: i32,

  // For partial write operations
  pub rb_roundtrips: usize, // How many times we went around the ringbuffer
  pub partial_pos_out: usize, // How much output to the user in total (<= rb)

  // For ReadHuffmanCode
  pub symbol: u32,
  pub repeat: u32,
  pub space: u32,

  pub table: [HuffmanCode; 32],
  // List of of symbol chains.
  pub symbol_lists_index: usize, // AllocU16::AllocatedMemory,
  // Storage from symbol_lists.
  pub symbols_lists_array: [u16; BROTLI_HUFFMAN_MAX_CODE_LENGTH + 1 +
                                 BROTLI_HUFFMAN_MAX_CODE_LENGTHS_SIZE],
  // Tails of symbol chains.
  pub next_symbol: [i32; 32],
  pub code_length_code_lengths: [u8; 18],
  // Population counts for the code lengths
  pub code_length_histo: [u16; 16],

  // For HuffmanTreeGroupDecode
  pub htree_index: i32,
  pub htree_next_offset: u32,

  // For DecodeContextMap
  pub context_index: u32,
  pub max_run_length_prefix: u32,
  pub code: u32,
  // always pre-allocated on state creation
  pub context_map_table: AllocHC::AllocatedMemory,

  // For InverseMoveToFrontTransform
  pub mtf_upper_bound: u32,
  pub mtf_or_error_string: Result<[u8; 256], [u8; 256]>,

  // For custom dictionaries
  pub custom_dict: AllocU8::AllocatedMemory,
  pub custom_dict_size: i32,
  // less used attributes are in the end of this struct */
  // States inside function calls
  pub substate_metablock_header: BrotliRunningMetablockHeaderState,
  pub substate_tree_group: BrotliRunningTreeGroupState,
  pub substate_context_map: BrotliRunningContextMapState,
  pub substate_uncompressed: BrotliRunningUncompressedState,
  pub substate_huffman: BrotliRunningHuffmanState,
  pub substate_decode_uint8: BrotliRunningDecodeUint8State,

  pub is_last_metablock: u8,
  pub is_uncompressed: u8,
  pub is_metadata: u8,
  pub size_nibbles: u8,
  pub window_bits: u32,

  pub num_literal_htrees: u32,
  pub context_map: AllocU8::AllocatedMemory,
  pub context_modes: AllocU8::AllocatedMemory,
  pub trivial_literal_contexts: [u32; 8],
}
macro_rules! make_brotli_state {
 ($alloc_u8 : expr, $alloc_u32 : expr, $alloc_hc : expr, $custom_dict : expr, $custom_dict_len: expr) => (BrotliState::<AllocU8, AllocU32, AllocHC>{
            state : BrotliRunningState::BROTLI_STATE_UNINITED,
            loop_counter : 0,
            br : BrotliBitReader::default(),
            alloc_u8 : $alloc_u8,
            alloc_u32 : $alloc_u32,
            alloc_hc : $alloc_hc,
            buffer : [0u8; 8],
            buffer_length : 0,
            pos : 0,
            max_backward_distance : 0,
            max_backward_distance_minus_custom_dict_size : 0,
            max_distance : 0,
            ringbuffer_size : 0,
            ringbuffer_mask: 0,
            dist_rb_idx : 0,
            dist_rb : [16, 15, 11, 4],
            ringbuffer : AllocU8::AllocatedMemory::default(),
            htree_command_index : 0,
            context_lookup : &kContextLookup[0],
            context_map_slice_index : 0,
            dist_context_map_slice_index : 0,
            sub_loop_counter : 0,

            literal_hgroup : HuffmanTreeGroup::<AllocU32, AllocHC>::default(),
            insert_copy_hgroup : HuffmanTreeGroup::<AllocU32, AllocHC>::default(),
            distance_hgroup : HuffmanTreeGroup::<AllocU32, AllocHC>::default(),
            trivial_literal_context : 0,
            distance_context : 0,
            meta_block_remaining_len : 0,
            block_type_length_state : BlockTypeAndLengthState::<AllocHC> {
              block_length_index : 0,
              block_length : [0; 3],
              num_block_types : [0;3],
              block_type_rb: [0;6],
              substate_read_block_length : BrotliRunningReadBlockLengthState::BROTLI_STATE_READ_BLOCK_LENGTH_NONE,
              block_type_trees : AllocHC::AllocatedMemory::default(),
              block_len_trees : AllocHC::AllocatedMemory::default(),
            },
            distance_postfix_bits : 0,
            num_direct_distance_codes : 0,
            distance_postfix_mask : 0,
            num_dist_htrees : 0,
            dist_context_map : AllocU8::AllocatedMemory::default(),
            //// not needed literal_htree : AllocHC::AllocatedMemory::default(),
            literal_htree_index : 0,
            dist_htree_index : 0,
            repeat_code_len : 0,
            prev_code_len : 0,
            copy_length : 0,
            distance_code : 0,
            rb_roundtrips : 0,  /* How many times we went around the ringbuffer */
            partial_pos_out : 0,  /* How much output to the user in total (<= rb) */
            symbol : 0,
            repeat : 0,
            space : 0,
            table : [HuffmanCode::default(); 32],
            //symbol_lists: AllocU16::AllocatedMemory::default(),
            symbol_lists_index : BROTLI_HUFFMAN_MAX_CODE_LENGTH + 1,
            symbols_lists_array : [0;BROTLI_HUFFMAN_MAX_CODE_LENGTH + 1 +
                              BROTLI_HUFFMAN_MAX_CODE_LENGTHS_SIZE],
            next_symbol : [0; 32],
            code_length_code_lengths : [0; 18],
            code_length_histo : [0; 16],
            htree_index : 0,
            htree_next_offset : 0,

            /* For DecodeContextMap */
           context_index : 0,
           max_run_length_prefix : 0,
           code : 0,
           context_map_table : AllocHC::AllocatedMemory::default(),

           /* For InverseMoveToFrontTransform */
           mtf_upper_bound : 255,
           mtf_or_error_string : Ok([0; 256]),

           /* For custom dictionaries */
           custom_dict : $custom_dict,
           custom_dict_size : $custom_dict_len as i32,

           /* less used attributes are in the end of this struct */
           /* States inside function calls */
           substate_metablock_header : BrotliRunningMetablockHeaderState::BROTLI_STATE_METABLOCK_HEADER_NONE,
           substate_tree_group : BrotliRunningTreeGroupState::BROTLI_STATE_TREE_GROUP_NONE,
           substate_context_map : BrotliRunningContextMapState::BROTLI_STATE_CONTEXT_MAP_NONE,
           substate_uncompressed : BrotliRunningUncompressedState::BROTLI_STATE_UNCOMPRESSED_NONE,
           substate_huffman : BrotliRunningHuffmanState::BROTLI_STATE_HUFFMAN_NONE,
           substate_decode_uint8 : BrotliRunningDecodeUint8State::BROTLI_STATE_DECODE_UINT8_NONE,

           is_last_metablock : 0,
           is_uncompressed : 0,
           is_metadata : 0,
           size_nibbles : 0,
           window_bits : 0,
           large_window: false,
           should_wrap_ringbuffer: false,
           error_code: BrotliDecoderErrorCode::BROTLI_DECODER_SUCCESS,
           num_literal_htrees : 0,
           context_map : AllocU8::AllocatedMemory::default(),
           context_modes : AllocU8::AllocatedMemory::default(),
           trivial_literal_contexts : [0u32; 8],
        }
    );
}
impl <'brotli_state,
      AllocU8 : alloc::Allocator<u8>,
      AllocU32 : alloc::Allocator<u32>,
      AllocHC : alloc::Allocator<HuffmanCode> > BrotliState<AllocU8, AllocU32, AllocHC> {
    pub fn new(alloc_u8 : AllocU8,
           alloc_u32 : AllocU32,
           alloc_hc : AllocHC) -> Self{
        let mut retval = make_brotli_state!(alloc_u8, alloc_u32, alloc_hc, AllocU8::AllocatedMemory::default(), 0);
        retval.large_window = true;
        retval.context_map_table = retval.alloc_hc.alloc_cell(
          BROTLI_HUFFMAN_MAX_TABLE_SIZE as usize);
        BrotliInitBitReader(&mut retval.br);
        retval
    }
    pub fn new_with_custom_dictionary(alloc_u8 : AllocU8,
           alloc_u32 : AllocU32,
           alloc_hc : AllocHC,
           custom_dict: AllocU8::AllocatedMemory) -> Self{
        let custom_dict_len = custom_dict.slice().len();
        let mut retval = make_brotli_state!(alloc_u8, alloc_u32, alloc_hc, custom_dict, custom_dict_len);
        retval.context_map_table = retval.alloc_hc.alloc_cell(
          BROTLI_HUFFMAN_MAX_TABLE_SIZE as usize);
        retval.large_window =  true;
        BrotliInitBitReader(&mut retval.br);
        retval
    }
    pub fn new_strict(alloc_u8 : AllocU8,
           alloc_u32 : AllocU32,
           alloc_hc : AllocHC) -> Self{
        let mut retval = make_brotli_state!(alloc_u8, alloc_u32, alloc_hc, AllocU8::AllocatedMemory::default(), 0);
        retval.context_map_table = retval.alloc_hc.alloc_cell(
          BROTLI_HUFFMAN_MAX_TABLE_SIZE as usize);
        retval.large_window =  false;
        BrotliInitBitReader(&mut retval.br);
        retval
    }
    pub fn BrotliStateMetablockBegin(self : &mut Self) {
        self.meta_block_remaining_len = 0;
        self.block_type_length_state.block_length[0] = 1u32 << 24;
        self.block_type_length_state.block_length[1] = 1u32 << 24;
        self.block_type_length_state.block_length[2] = 1u32 << 24;
        self.block_type_length_state.num_block_types[0] = 1;
        self.block_type_length_state.num_block_types[1] = 1;
        self.block_type_length_state.num_block_types[2] = 1;
        self.block_type_length_state.block_type_rb[0] = 1;
        self.block_type_length_state.block_type_rb[1] = 0;
        self.block_type_length_state.block_type_rb[2] = 1;
        self.block_type_length_state.block_type_rb[3] = 0;
        self.block_type_length_state.block_type_rb[4] = 1;
        self.block_type_length_state.block_type_rb[5] = 0;
        self.alloc_u8.free_cell(core::mem::replace(&mut self.context_map,
                                             AllocU8::AllocatedMemory::default()));
        self.alloc_u8.free_cell(core::mem::replace(&mut self.context_modes,
                                             AllocU8::AllocatedMemory::default()));
        self.alloc_u8.free_cell(core::mem::replace(&mut self.dist_context_map,
                                             AllocU8::AllocatedMemory::default()));
        self.context_map_slice_index = 0;
        self.literal_htree_index = 0;
        self.dist_context_map_slice_index = 0;
        self.dist_htree_index = 0;
        self.context_lookup = &kContextLookup[0];
        self.literal_hgroup.reset(&mut self.alloc_u32, &mut self.alloc_hc);
        self.insert_copy_hgroup.reset(&mut self.alloc_u32, &mut self.alloc_hc);
        self.distance_hgroup.reset(&mut self.alloc_u32, &mut self.alloc_hc);
    }
    pub fn BrotliStateCleanupAfterMetablock(self : &mut Self) {
        self.alloc_u8.free_cell(core::mem::replace(&mut self.context_map,
                                             AllocU8::AllocatedMemory::default()));
        self.alloc_u8.free_cell(core::mem::replace(&mut self.context_modes,
                                             AllocU8::AllocatedMemory::default()));
        self.alloc_u8.free_cell(core::mem::replace(&mut self.dist_context_map,
                                             AllocU8::AllocatedMemory::default()));


        self.literal_hgroup.reset(&mut self.alloc_u32, &mut self.alloc_hc);
        self.insert_copy_hgroup.reset(&mut self.alloc_u32, &mut self.alloc_hc);
        self.distance_hgroup.reset(&mut self.alloc_u32, &mut self.alloc_hc);
    }

   fn BrotliStateCleanup(self : &mut Self) {
      self.BrotliStateCleanupAfterMetablock();
      self.alloc_u8.free_cell(core::mem::replace(&mut self.ringbuffer,
                              AllocU8::AllocatedMemory::default()));
      self.alloc_hc.free_cell(core::mem::replace(&mut self.block_type_length_state.block_type_trees,
                              AllocHC::AllocatedMemory::default()));
      self.alloc_hc.free_cell(core::mem::replace(&mut self.block_type_length_state.block_len_trees,
                              AllocHC::AllocatedMemory::default()));
      self.alloc_hc.free_cell(core::mem::replace(&mut self.context_map_table,
                              AllocHC::AllocatedMemory::default()));
      self.alloc_u8.free_cell(core::mem::replace(&mut self.custom_dict,
                              AllocU8::AllocatedMemory::default()));

      //FIXME??  BROTLI_FREE(s, s->legacy_input_buffer);
      //FIXME??  BROTLI_FREE(s, s->legacy_output_buffer);
    }

    pub fn BrotliStateIsStreamStart(self : &Self) -> bool {
        match self.state {
            BrotliRunningState::BROTLI_STATE_UNINITED =>
                BrotliGetAvailableBits(&self.br) == 0,
            _ => false,
        }
    }

    pub fn BrotliStateIsStreamEnd(self : &Self) -> bool {
        match self.state {
            BrotliRunningState::BROTLI_STATE_DONE => true,
            _ => false
        }
    }
    pub fn BrotliHuffmanTreeGroupInit(self :&mut Self, group : WhichTreeGroup,
                                      alphabet_size : u16, max_symbol: u16, ntrees : u16) {
        match group {
            WhichTreeGroup::LITERAL => self.literal_hgroup.init(&mut self.alloc_u32,
                                                                &mut self.alloc_hc,
                                                                alphabet_size, max_symbol, ntrees),
            WhichTreeGroup::INSERT_COPY => self.insert_copy_hgroup.init(&mut self.alloc_u32,
                                                                        &mut self.alloc_hc,
                                                                        alphabet_size, max_symbol, ntrees),
            WhichTreeGroup::DISTANCE => self.distance_hgroup.init(&mut self.alloc_u32,
                                                                  &mut self.alloc_hc,
                                                                  alphabet_size, max_symbol, ntrees),
        }
    }
    pub fn BrotliHuffmanTreeGroupRelease(self :&mut Self, group : WhichTreeGroup) {
        match group {
            WhichTreeGroup::LITERAL => self.literal_hgroup.reset(&mut self.alloc_u32,
                                                                 &mut self.alloc_hc),
            WhichTreeGroup::INSERT_COPY => self.insert_copy_hgroup.reset(&mut self.alloc_u32,
                                                                         &mut self.alloc_hc),
            WhichTreeGroup::DISTANCE => self.distance_hgroup.reset(&mut self.alloc_u32,
                                                                   &mut self.alloc_hc),
        }
    }
}

impl <'brotli_state,
      AllocU8 : alloc::Allocator<u8>,
      AllocU32 : alloc::Allocator<u32>,
      AllocHC : alloc::Allocator<HuffmanCode> > Drop for BrotliState<AllocU8, AllocU32, AllocHC> {
    fn drop(&mut self) {
        self.BrotliStateCleanup();
    }
}



pub fn BrotliDecoderErrorStr(c: BrotliDecoderErrorCode) -> &'static str {
  match c {
  BrotliDecoderErrorCode::BROTLI_DECODER_NO_ERROR => "NO_ERROR\0",
  /* Same as BrotliDecoderResult values */
  BrotliDecoderErrorCode::BROTLI_DECODER_SUCCESS => "SUCCESS\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_NEEDS_MORE_INPUT => "NEEDS_MORE_INPUT\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_NEEDS_MORE_OUTPUT => "NEEDS_MORE_OUTPUT\0",

  /* Errors caused by invalid input */
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_NIBBLE => "ERROR_FORMAT_EXUBERANT_NIBBLE\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_RESERVED => "ERROR_FORMAT_RESERVED\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_EXUBERANT_META_NIBBLE => "ERROR_FORMAT_EXUBERANT_META_NIBBLE\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_ALPHABET => "ERROR_FORMAT_SIMPLE_HUFFMAN_ALPHABET\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_SIMPLE_HUFFMAN_SAME => "ERROR_FORMAT_SIMPLE_HUFFMAN_SAME\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_CL_SPACE => "ERROR_FORMAT_FL_SPACE\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_HUFFMAN_SPACE => "ERROR_FORMAT_HUFFMAN_SPACE\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_CONTEXT_MAP_REPEAT => "ERROR_FORMAT_CONTEXT_MAP_REPEAT\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_1 =>"ERROR_FORMAT_BLOCK_LENGTH_1\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_BLOCK_LENGTH_2 =>"ERROR_FORMAT_BLOCK_LENGTH_2\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_TRANSFORM => "ERROR_FORMAT_TRANSFORM\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_DICTIONARY =>"ERROR_FORMAT_DICTIONARY\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_WINDOW_BITS =>"ERROR_FORMAT_WINDOW_BITS\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_PADDING_1 =>"ERROR_FORMAT_PADDING_1\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_PADDING_2 =>"ERROR_FORMAT_PADDING_2\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_FORMAT_DISTANCE =>"ERROR_FORMAT_DISTANCE\0",

  /* -17..-18 codes are reserved */

  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_DICTIONARY_NOT_SET => "ERROR_DICTIONARY_NOT_SET\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_INVALID_ARGUMENTS => "ERROR_INVALID_ARGUMENTS\0",

  /* Memory allocation problems */
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MODES => "ERROR_ALLOC_CONTEXT_MODES\0",
  /* Literal => insert and distance trees together */
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_ALLOC_TREE_GROUPS => "ERROR_ALLOC_TREE_GROUPS\0",
  /* -23..-24 codes are reserved for distinct tree groups */
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_ALLOC_CONTEXT_MAP => "ERROR_ALLOC_CONTEXT_MAP\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_1 => "ERROR_ALLOC_RING_BUFFER_1\0",
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_ALLOC_RING_BUFFER_2 => "ERROR_ALLOC_RING_BUFFER_2\0",
  /* -28..-29 codes are reserved for dynamic ring-buffer allocation */
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_ALLOC_BLOCK_TYPE_TREES => "ERROR_ALLOC_BLOCK_TYPE_TREES\0",

  /* "Impossible" states */
  BrotliDecoderErrorCode::BROTLI_DECODER_ERROR_UNREACHABLE => "ERROR_UNREACHABLE\0",
  }
}
