--[[
    This is a dissector for HOPR protocol stack.
    It however does not dissect raw network packets, but works only with custom
    diagnostic capture format enabled using the "capture" feature on this crate.

    Installation:
        mkdir -p $HOME/.local/lib/wireshark/plugins/
        cp hopr.lua $HOME/.local/lib/wireshark/plugins/
--]]

-- HOPR Start Protocol Lua dissector

local hopr_start = Proto("hopr_start", "HOPR Start Protocol")

local start_msg_types = {
    [0x00] = "StartSession",
    [0x01] = "SessionEstablished",
    [0x02] = "SessionError",
    [0x03] = "CloseSession",
    [0x04] = "KeepAlive"
}

-- Start protocol fields
local start_fields = {
    version = ProtoField.uint8("hopr_start.version", "Version", base.DEC),
    type = ProtoField.uint8("hopr_start.type", "Type", base.HEX, start_msg_types),

    msg = ProtoField.bytes("hopr_start.message", "Bincode encoded message")
}

hopr_start.fields = start_fields

-- Dissector function
local function dissect_hopr_start(buffer, pinfo, tree)
    local subtree = tree:add(hopr_start, buffer())

    local offset = 0
    subtree:add(start_fields.version, buffer(offset,1))
    local version = buffer(offset,1):uint()

    if version ~= 0x01 then
        subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Unsupported Start version " .. version )
        return offset
    end

    offset = offset + 1

    subtree:add(start_fields.type, buffer(offset,1))
    local type = buffer(offset, 1):uint()
    pinfo.cols.info:append(", " .. start_msg_types[type] or "Unknown")
    offset = offset + 1

    subtree:add(start_fields.msg, buffer(offset))

    return offset
end

---------------------------------------------------------------------------------------

-- HOPR Probe Protocol Lua dissector

local hopr_probe = Proto("hopr_probe", "HOPR Probe Protocol")

local probe_types = {
    [0x00] = "Telemetry",
    [0x01] = "Probe",
}

local probe_probe_types = {
    [0x00] = "Ping",
    [0x01] = "Pong",
}

-- Start protocol fields
local probe_fields = {
    version = ProtoField.uint8("hopr_probe.version", "Version", base.DEC),
    type = ProtoField.uint8("hopr_probe.type", "Type", base.HEX, probe_types),

    probe_type = ProtoField.uint8("hopr_probe.probe.type", "Probe Type", base.HEX, probe_probe_types),
    probe_nonce = ProtoField.bytes("hopr_probe.probe.nonce", "Probe Nonce"),

    tele_id = ProtoField.bytes("hopr_probe.telemetry.id", "Telemetry ID"),
    tele_path = ProtoField.bytes("hopr_probe.telemetry.path", "Telemetry Path"),
    tele_ts = ProtoField.bytes("hopr_probe.telemetry.ts", "Telemetry Timestamp"),
}

hopr_probe.fields = probe_fields

-- Dissector function
local function dissect_hopr_probe(buffer, pinfo, tree)
    local subtree = tree:add(hopr_probe, buffer())

    local offset = 0
    subtree:add(probe_fields.version, buffer(offset,1))
    local version = buffer(offset,1):uint()

    if version ~= 0x01 then
        subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Unsupported Probe version " .. version )
        return offset
    end

    offset = offset + 1

    subtree:add(probe_fields.type, buffer(offset,1))
    local msg_type = buffer(offset,1):uint()
    offset = offset + 1

    if msg_type == 0x00 then -- Telemetry
        pinfo.cols.info:append(", Telemetry")

        local tele_tree = subtree:add("Telemetry")
        tele_tree:add(probe_fields.tele_id, buffer(offset, 10))
        offset = offset + 10

        tele_tree:add(probe_fields.tele_path, buffer(offset, 10))
        offset = offset + 10

        tele_tree:add(probe_fields.tele_ts, buffer(offset, 16))
        offset = offset + 16
    elseif msg_type == 0x01 then -- Probe

        local probe_tree = subtree:add("Probe")
        probe_tree:add(probe_fields.probe_type, buffer(offset, 1))

        local probe_type = buffer(offset,1):uint()
        pinfo.cols.info:append(", " .. probe_probe_types[probe_type] or "Unknown")

        offset = offset + 1

        probe_tree:add(probe_fields.probe_nonce, buffer(offset, 32))
        offset = offset + 32
    else
        subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Unknown Probe message type: " .. msg_type)
    end

    return offset
end

---------------------------------------------------------------------------------------

-- HOPR Session Protocol Lua dissector

local hopr_session = Proto("hopr_session", "HOPR Session Protocol")

-- Session protocol fields
local session_fields = {
    version = ProtoField.uint8("hopr_session.version", "Version", base.DEC),
    
    type = ProtoField.uint8("hopr_session.type", "Type", base.HEX, {
        [0x00] = "Segment",
        [0x01] = "SegmentRequest",
        [0x02] = "FrameAcknowledgements"
    }),

    len = ProtoField.uint16("hopr_session.len", "Message Length", base.DEC),

    -- Segment fields
    seg_frame_id = ProtoField.framenum("hopr_session.segment.frame_id", "Frame ID", base.NONE, frametype.NONE),
    seg_idx = ProtoField.uint8("hopr_session.segment.seg_idx", "Segment Index", base.DEC),
    seg_terminating = ProtoField.bool("hopr_session.segment.terminating", "Terminating", 8, nil, 0x80),
    seg_seq_len = ProtoField.uint8("hopr_session.segment.seq_len", "Sequence Length", base.DEC, nil, 0x3f),
    seg_data = ProtoField.bytes("hopr_session.segment.data", "Data"),

    -- SegmentRequest fields
    req_frame_id = ProtoField.framenum("hopr_session.segment_request.frame_id", "Frame ID", base.NONE, frametype.REQUEST),
    req_missing_1 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_1", "Segment 1 missing", 8, nil, 0x80),
    req_missing_2 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_2", "Segment 2 missing", 8, nil, 0x40),
    req_missing_3 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_3", "Segment 3 missing", 8, nil, 0x20),
    req_missing_4 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_4", "Segment 4 missing", 8, nil, 0x10),
    req_missing_5 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_5", "Segment 5 missing", 8, nil, 0x08),
    req_missing_6 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_6", "Segment 6 missing", 8, nil, 0x04),
    req_missing_7 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_7", "Segment 7 missing", 8, nil, 0x02),
    req_missing_8 = ProtoField.bool("hopr_session.segment_request.missing_segments.seg_8", "Segment 8 missing", 8, nil, 0x01),

    -- FrameAcknowledgement fields
    ack_frame_id = ProtoField.framenum("hopr_session.frame_ack.frame_id", "Frame ID", base.NONE, frametype.ACK)
}

hopr_session.fields = session_fields

-- Dissector function
local function dissect_hopr_session(buffer, pinfo, tree)
    local subtree = tree:add(hopr_session, buffer())

    local offset = 0
    subtree:add(session_fields.version, buffer(offset,1))
    local version = buffer(offset,1):uint()
    offset = offset + 1

    subtree:add(session_fields.type, buffer(offset,1))
    local msg_type = buffer(offset,1):uint()
    offset = offset + 1

    subtree:add(session_fields.len, buffer(offset,2))
    local msg_len = buffer(offset,2):uint()
    offset = offset + 2

    -- Parse message based on type
    if msg_type == 0x00 then
        -- Segment
        local frame_id = buffer(offset,4):uint()
        local seg_idx = buffer(offset+4,1):uint()
        pinfo.cols.info:append(", Segment (" .. frame_id .. "," ..seg_idx.. ")")
        local seg_tree = subtree:add("Segment")
        seg_tree:add(session_fields.seg_frame_id, frame_id)
        seg_tree:add(session_fields.seg_idx, seg_idx)
        local seg_flags = seg_tree:add("Sequence flags")
        seg_flags:add(session_fields.seg_terminating, buffer(offset+5,1))
        seg_flags:add(session_fields.seg_seq_len, buffer(offset+5,1))
        local data_len = msg_len - 6
        local data_buf = buffer(offset+6, data_len)

        -- Call heuristic dissectors on Segment.data
        -- This allows Wireshark to attempt to decode the payload inside Segment.data
        local data_tvb = data_buf:tvb()

        -- Get the heuristic dissector table for "data"
        succ = DissectorTable.try_heuristics("udp", data_tvb, pinfo, seg_tree)
        if not succ then
            succ = DissectorTable.try_heuristics("tcp", data_tvb, pinfo, seg_tree)
            if not succ then
                -- Fallback: just add raw bytes if no heuristic table found
                seg_tree:add(session_fields.seg_data, data_buf)
            end
        end
    elseif msg_type == 0x01 then
        -- SegmentRequest[]
        local end_offset = offset + msg_len
        local idx = 0
        while offset < end_offset do
            local frame_id = buffer(offset,4):uint()
            if frame_id == 0 then 
                break 
            end

            local req_tree = subtree:add("SegmentRequest["..idx.."]")
            req_tree:add(session_fields.req_frame_id, frame_id)
            local missing = req_tree:add("Missing segments")
            missing:add(session_fields.req_missing_1, buffer(offset+4,1))
            missing:add(session_fields.req_missing_2, buffer(offset+4,1))
            missing:add(session_fields.req_missing_3, buffer(offset+4,1))
            missing:add(session_fields.req_missing_4, buffer(offset+4,1))
            missing:add(session_fields.req_missing_5, buffer(offset+4,1))
            missing:add(session_fields.req_missing_6, buffer(offset+4,1))
            missing:add(session_fields.req_missing_7, buffer(offset+4,1))
            missing:add(session_fields.req_missing_8, buffer(offset+4,1))
            offset = offset + 5
            idx = idx + 1
        end
        pinfo.cols.info:append(", SegmentRequest ("..idx..")")
    elseif msg_type == 0x02 then
        -- FrameAcknowledgement[]
        local end_offset = offset + msg_len
        local idx = 0
        while offset < end_offset do
            local frame_id = buffer(offset,4):uint()
            if frame_id == 0 then 
                break
            end

            local ack_tree = subtree:add("FrameAcknowledgement["..idx.."]")
            ack_tree:add(session_fields.ack_frame_id, frame_id)
            offset = offset + 4
            idx = idx + 1
        end
        pinfo.cols.info:append(", FrameAcknowledgements ("..idx..")")
    end

    return 2 + msg_len
end

function hopr_session.dissector(buffer, pinfo, tree)
    local length = buffer:len()
    if length < 1 then return end

    pinfo.cols.protocol = "HOPR Session"
    pinfo.cols.info:append("Session")
    return dissect_hopr_session(buffer, pinfo, tree)
end

---------------------------------------------------------------------------------------

-- HOPR Protocol Wireshark Lua Dissector

local hopr_proto = Proto("hopr", "HOPR Protocol")


-- HOPR Protocol fields
local hopr_fields = {
    type = ProtoField.uint8("hopr.type", "Packet Type", base.DEC, {
        [0x00] = "Final",
        [0x01] = "Forwarded",
        [0x02] = "Outgoing",
        [0x03] = "AcknowledgementIn",
        [0x04] = "AcknowledgementOut",
    }),

    -- Common fields
    packet_tag = ProtoField.bytes("hopr.packet_tag", "Packet Tag"),
    previous_hop = ProtoField.bytes("hopr.previous_hop", "Previous Hop"),
    previous_hop_peer_id = ProtoField.stringz("hopr.previous_hop_peer_id", "Previous Hop (Peer ID)"),
    next_hop = ProtoField.bytes("hopr.next_hop", "Next Hop"),
    next_hop_peer_id = ProtoField.stringz("hopr.next_hop_peer_id", "Next Hop (Peer ID)"),
    data_len = ProtoField.uint16("hopr.data_len", "Data Length"),

    -- Ticket fields
    ticket_channel_id = ProtoField.bytes("hopr.ticket.channel_id", "Channel ID"),
    ticket_amount = ProtoField.bytes("hopr.ticket.amount", "Amount"),
    ticket_index = ProtoField.uint64("hopr.ticket.index", "Index"),
    ticket_offset = ProtoField.uint32("hopr.ticket.index_offset", "Index offset"),
    ticket_epoch = ProtoField.uint24("hopr.ticket.channel_epoch", "Channel epoch"),
    ticket_challenge = ProtoField.bytes("hopr.ticket.challenge", "Ethereum challenge"),
    ticket_luck = ProtoField.uint64("hopr.ticket.luck", "Luck"),
    ticket_win_prob = ProtoField.double("hopr.ticket.win_prob", "Winning probability"),
    ticket_signature = ProtoField.bytes("hopr.ticket.signature", "Signature"),

    -- FinalPacket specific
    sender_pseudonym = ProtoField.bytes("hopr.sender_pseudonym", "Sender Pseudonym"),

    ack_key = ProtoField.bytes("hopr.ack.key", "ACK Key"),
    ack_sig = ProtoField.bytes("hopr.ack.signature", "Signature"),
    challenge = ProtoField.bytes("hopr.challenge", "Challenge"),

    -- ApplicationData
    appdata_tag = ProtoField.uint64("hopr.appdata.tag", "Tag", base.HEX),
    appdata_type = ProtoField.uint8("hopr.appdata.type", "Type", base.DEC, {
        [0x00] = "Probe",
        [0x01] = "Start",
        [0x0f] = "Undefined",
        [0x10] = "Session"
    }),
    appdata_data = ProtoField.bytes("hopr.appdata.data", "Data")
}

hopr_proto.fields = hopr_fields

-- Helper function to convert IEEE 754 bits to double
local function convert_ieee_to_double(high32, low32)
    -- Create byte array in big-endian order
    local bytes = {}
    bytes[1] = bit.rshift(high32, 24)
    bytes[2] = bit.band(bit.rshift(high32, 16), 0xFF)
    bytes[3] = bit.band(bit.rshift(high32, 8), 0xFF)
    bytes[4] = bit.band(high32, 0xFF)
    bytes[5] = bit.rshift(low32, 24)
    bytes[6] = bit.band(bit.rshift(low32, 16), 0xFF)
    bytes[7] = bit.band(bit.rshift(low32, 8), 0xFF)
    bytes[8] = bit.band(low32, 0xFF)

    -- Create ByteArray and convert to Tvb
    local ba = ByteArray.new()
    ba:set_size(8)
    for i = 1, 8 do
        ba:set_index(i - 1, bytes[i])
    end

    -- Convert to Tvb and extract as double
    local tvb = ba:tvb("IEEE754")
    return tvb(0, 8):float()
end

local function luck_to_double(buffer)
    -- Input validation
    if buffer:len() ~= 7 then
        error("Input must be exactly 7 bytes")
    end

    -- Check for special case: all zeros
    local all_zeros = true
    for i = 0, 6 do
        if buffer(i, 1):uint() ~= 0 then
            all_zeros = false
            break
        end
    end
    if all_zeros then
        return 0.0
    end

    -- Check for special case: all 0xff
    local all_ff = true
    for i = 0, 6 do
        if buffer(i, 1):uint() ~= 0xff then
            all_ff = false
            break
        end
    end
    if all_ff then
        return 1.0
    end

    -- Build 64-bit value from 7 bytes (big-endian)
    local high32 = 0
    local low32 = 0

    -- Process first 3 bytes for high 32 bits
    for i = 0, 2 do
        high32 = high32 + bit.lshift(buffer(i, 1):uint(), 8 * (2 - i))
    end

    -- Process last 4 bytes for low 32 bits
    for i = 3, 6 do
        low32 = low32 + bit.lshift(buffer(i, 1):uint(), 8 * (6 - i))
    end

    -- Add 1 to the 56-bit value (significand = tmp + 1)
    low32 = low32 + 1
    if low32 >= 2^32 then
        high32 = high32 + 1
        low32 = low32 - 2^32
    end

    -- Right shift by 4 bits (significand >> 4)
    local shifted_low = bit.rshift(low32, 4)
    local shifted_high = bit.rshift(high32, 4)
    shifted_low = shifted_low + bit.lshift(bit.band(high32, 0xF), 28)

    -- Add IEEE 754 exponent bias (1023 << 52)
    shifted_high = shifted_high + bit.lshift(1023, 20)  -- 52 - 32 = 20

    -- Convert to double using byte manipulation
    return convert_ieee_to_double(shifted_high, shifted_low) - 1.0
end


local function dissect_ticket(buffer, tree, offset)
    local ticket_tree = tree:add("Ticket")

    local data_len = buffer(offset, 1):uint()
    offset = offset + 1
    if data_len ~= 148 then
        ticket_tree:add_expert_info(PI_MALFORMED, PI_ERROR, "Invalid ticket length: ".. data_len)
        return offset
    end

    ticket_tree:add(hopr_fields.ticket_channel_id, buffer(offset, 32))
    offset = offset + 32

    ticket_tree:add(hopr_fields.ticket_amount, buffer(offset, 12))
    offset = offset + 12

    ticket_tree:add(hopr_fields.ticket_index, buffer(offset, 6):uint64())
    offset = offset + 6

    ticket_tree:add(hopr_fields.ticket_offset, buffer(offset, 4):uint())
    offset = offset + 4

    ticket_tree:add(hopr_fields.ticket_epoch, buffer(offset, 3):uint())
    offset = offset + 3

    ticket_tree:add(hopr_fields.ticket_luck, buffer(offset, 7))
    ticket_tree:add(hopr_fields.ticket_win_prob, luck_to_double(buffer(offset, 7)))
    offset = offset + 7

    ticket_tree:add(hopr_fields.ticket_challenge, buffer(offset, 20))
    offset = offset + 20

    ticket_tree:add(hopr_fields.ticket_signature, buffer(offset, 64))
    offset = offset + 64

    return offset
end

-- ApplicationData dissector
local function dissect_appdata(buffer, tree, offset, data_len, pinfo)
    local appdata_tree = tree:add("ApplicationData")

    -- Tag (u64)
    local tag_field = buffer(offset, 8)
    local tag = tag_field:uint64()
    appdata_tree:add(hopr_fields.appdata_tag, tag_field)
    offset = offset + 8

    -- Data (variable length)
    if data_len > 8 then
        local data_field = buffer(offset, data_len - 8)
        if tag == UInt64(0) then
            pinfo.cols.info:append(", Probe")
            appdata_tree:add(hopr_fields.appdata_type, 0)
            dissect_hopr_probe(data_field, pinfo, appdata_tree)
        elseif tag == UInt64(1) then
            pinfo.cols.info:append(", Start")
            appdata_tree:add(hopr_fields.appdata_type, 1)
            dissect_hopr_start(data_field, pinfo, appdata_tree)
        elseif tag >= UInt64(16) then
            pinfo.cols.info:append(", Session")
            appdata_tree:add(hopr_fields.appdata_type, 16)
            dissect_hopr_session(data_field, pinfo, appdata_tree)
        else
            pinfo.cols.info:append(", Unknown application tag")
            appdata_tree:add(hopr_fields.appdata_type, 15)
            appdata_tree:add(hopr_fields.appdata_data, data_field)
        end
        offset = offset + data_len - 8
    end

    return offset
end

-- Main dissector
function hopr_proto.dissector(buffer, pinfo, tree)
    local length = buffer:len()
    if length < 1 then return end

    pinfo.cols.protocol = "HOPR"
    local offset = 0
    local subtree = tree:add(hopr_proto, buffer(), "HOPR Protocol")

    -- Packet Type (u8)
    local pkt_type = buffer(offset, 1):uint()
    subtree:add(hopr_fields.type, buffer(offset, 1))
    offset = offset + 1

    -- Process based on packet type
    if pkt_type == 0 then -- FinalPacket
        if length < 1 + 16 + 32 + 32 + 10 + 32 + 2 + 8 then
            subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Packet too short for FinalPacket")
            return
        end
        pinfo.cols.info:set("Incoming")

        local final_tree = subtree:add("FinalPacket")
        final_tree:add(hopr_fields.packet_tag, buffer(offset, 16))
        offset = offset + 16

        final_tree:add(hopr_fields.previous_hop, buffer(offset, 32))
        offset = offset + 32

        local prev_hop_peer_id = buffer(offset):stringz()
        final_tree:add(hopr_fields.previous_hop_peer_id, prev_hop_peer_id)
        pinfo.cols.src = prev_hop_peer_id

        offset = offset + prev_hop_peer_id:len() + 1

        final_tree:add(hopr_fields.next_hop, buffer(offset, 32))
        offset = offset + 32

        local next_hop_peer_id = buffer(offset):stringz()
        final_tree:add(hopr_fields.next_hop_peer_id, next_hop_peer_id)
        pinfo.cols.dst = next_hop_peer_id

        offset = offset + next_hop_peer_id:len() + 1

        final_tree:add(hopr_fields.sender_pseudonym, buffer(offset, 10))
        offset = offset + 10

        final_tree:add(hopr_fields.ack_key, buffer(offset, 32))
        offset = offset + 32

        offset = dissect_ticket(buffer, final_tree, offset)

        local data_len_field = buffer(offset, 2)
        local data_len = data_len_field:uint()
        final_tree:add(hopr_fields.data_len, data_len_field)
        offset = offset + 2

        offset = dissect_appdata(buffer, final_tree, offset, data_len, pinfo)

    elseif pkt_type == 1 then -- ForwardedPacket
        if length < 1 + 16 + 32 + 32 + 96 then
            subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Packet too short for ForwardedPacket")
            return
        end
        pinfo.cols.info:set("Relayed")

        local fwd_tree = subtree:add("ForwardedPacket")
        fwd_tree:add(hopr_fields.packet_tag, buffer(offset, 16))
        offset = offset + 16

        fwd_tree:add(hopr_fields.previous_hop, buffer(offset, 32))
        offset = offset + 32

        local prev_hop_peer_id = buffer(offset):stringz()
        fwd_tree:add(hopr_fields.previous_hop_peer_id, prev_hop_peer_id)
        pinfo.cols.src = prev_hop_peer_id

        offset = offset + prev_hop_peer_id:len() + 1

        fwd_tree:add(hopr_fields.next_hop, buffer(offset, 32))
        offset = offset + 32

        local next_hop_peer_id = buffer(offset):stringz()
        fwd_tree:add(hopr_fields.next_hop_peer_id, next_hop_peer_id)
        pinfo.cols.dst = next_hop_peer_id

        offset = offset + next_hop_peer_id:len() + 1

        local ack_subtree = fwd_tree:add("Acknowledgement Data")
        ack_subtree:add(hopr_fields.ack_key, buffer(offset, 32))
        offset = offset + 32
        ack_subtree:add(hopr_fields.ack_sig, buffer(offset, 64))
        offset = offset + 64

        offset = dissect_ticket(buffer, fwd_tree, offset)
    elseif pkt_type == 2 then -- OutgoingPacket
        if length < 1 + 32 + 32 + 33 + 2 + 8 then
            subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Packet too short for OutgoingPacket")
            return
        end
        pinfo.cols.info:set("Outgoing")

        local out_tree = subtree:add("OutgoingPacket")

        out_tree:add(hopr_fields.previous_hop, buffer(offset, 32))
        offset = offset + 32

        local prev_hop_peer_id = buffer(offset):stringz()
        out_tree:add(hopr_fields.previous_hop_peer_id, prev_hop_peer_id)
        pinfo.cols.src = prev_hop_peer_id

        offset = offset + prev_hop_peer_id:len() + 1

        out_tree:add(hopr_fields.next_hop, buffer(offset, 32))
        offset = offset + 32

        local next_hop_peer_id = buffer(offset):stringz()
        out_tree:add(hopr_fields.next_hop_peer_id, next_hop_peer_id)
        pinfo.cols.dst = next_hop_peer_id

        offset = offset + next_hop_peer_id:len() + 1

        out_tree:add(hopr_fields.challenge, buffer(offset, 33))
        offset = offset + 33

        offset = dissect_ticket(buffer, out_tree, offset)

        local data_len_field = buffer(offset, 2)
        local data_len = data_len_field:uint()
        out_tree:add(hopr_fields.data_len, data_len_field)
        offset = offset + 2

        offset = dissect_appdata(buffer, out_tree, offset, data_len, pinfo)

    elseif pkt_type == 3 then -- AcknowledgementIn
        if length < 1 + 16 + 32 + 32 + 96 then
            subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Packet too short for AckIn")
            return
        end
        pinfo.cols.info:set("Incoming, Acknowledgement")

        local ack_in_tree = subtree:add("Acknowledgement")
        ack_in_tree:add(hopr_fields.packet_tag, buffer(offset, 16))
        offset = offset + 16

        ack_in_tree:add(hopr_fields.previous_hop, buffer(offset, 32))
        offset = offset + 32

        local prev_hop_peer_id = buffer(offset):stringz()
        ack_in_tree:add(hopr_fields.previous_hop_peer_id, prev_hop_peer_id)
        pinfo.cols.src = prev_hop_peer_id

        offset = offset + prev_hop_peer_id:len() + 1

        ack_in_tree:add(hopr_fields.next_hop, buffer(offset, 32))
        offset = offset + 32

        local next_hop_peer_id = buffer(offset):stringz()
        ack_in_tree:add(hopr_fields.next_hop_peer_id, next_hop_peer_id)
        pinfo.cols.dst = next_hop_peer_id

        offset = offset + next_hop_peer_id:len() + 1

        local ack_subtree = ack_in_tree:add("Acknowledgement Data")
        ack_subtree:add(hopr_fields.ack_key, buffer(offset, 32))
        offset = offset + 32
        ack_subtree:add(hopr_fields.ack_sig, buffer(offset, 64))
        offset = offset + 64

    elseif pkt_type == 4 then -- AcknowledgementOut
            if length < 1 + 32 + 1 + 96 then
                subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Packet too short for AckOut")
                return
            end
            pinfo.cols.info:set("Outgoing, Acknowledgement")

            local ack_out_tree = subtree:add("Acknowledgement")

            ack_out_tree:add(hopr_fields.previous_hop, buffer(offset, 32))
            offset = offset + 32

            local prev_hop_peer_id = buffer(offset):stringz()
            ack_out_tree:add(hopr_fields.previous_hop_peer_id, prev_hop_peer_id)
            pinfo.cols.src = prev_hop_peer_id

            offset = offset + prev_hop_peer_id:len() + 1

            ack_out_tree:add(hopr_fields.next_hop, buffer(offset, 32))
            offset = offset + 32

            local next_hop_peer_id = buffer(offset):stringz()
            ack_out_tree:add(hopr_fields.next_hop_peer_id, next_hop_peer_id)
            pinfo.cols.dst = next_hop_peer_id

            offset = offset + next_hop_peer_id:len() + 1

            if buffer(offset, 1):uint() == 1 then
                subtree:add_expert_info(PI_NOTE, PI_ERROR, "This acknowledgement is random due to processing error on the node")
            end
            offset = offset + 1

            local ack_subtree = ack_out_tree:add("Acknowledgement Data")
            ack_subtree:add(hopr_fields.ack_key, buffer(offset, 32))
            offset = offset + 32
            ack_subtree:add(hopr_fields.ack_sig, buffer(offset, 64))
            offset = offset + 64
    else
        subtree:add_expert_info(PI_MALFORMED, PI_ERROR, "Unknown packet type: " .. pkt_type)
    end
end

-- Register dissector
local ethertype_table = DissectorTable.get("ethertype")
ethertype_table:add(0x1234, hopr_proto)
ethertype_table:add(0x1235, hopr_session)