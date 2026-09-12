--[[
    Wireshark dissector for the HOPR protocol stack.

    This does not dissect raw network traffic. It reads the diagnostic capture format written by
    the `capture` feature of the `hopr-transport` crate: a pcapng with link type USER0 whose frames
    are the node's own dissection of each packet at the moment it entered or left the transport.
    See the module documentation of `transport/hopr/src/capture.rs` for the frame layout.

    Produce a capture with:
        HOPR_CAPTURE_PACKETS=/tmp/hopr.pcapng hoprd ...      # binary built with --features capture

    Install (Linux):
        mkdir -p $HOME/.local/lib/wireshark/plugins/
        cp transport/hopr/hopr.lua $HOME/.local/lib/wireshark/plugins/

    Install (macOS):
        mkdir -p ~/.config/wireshark/plugins
        cp transport/hopr/hopr.lua ~/.config/wireshark/plugins/

    Or load it for a single run:
        tshark -X lua_script:transport/hopr/hopr.lua -r /tmp/hopr.pcapng

    The block of constants below is generated from the Rust protocol definitions and kept in sync by
    `mod dissector` in `transport/hopr/src/capture.rs`. Everything below the block reads from it, so
    a protocol change is normally repaired by regenerating rather than by editing offsets by hand.
--]]

-- >>> BEGIN GENERATED WIRE CONSTANTS
-- Generated from the Rust protocol definitions; do not edit by hand.
-- Regenerate with:
--   HOPR_UPDATE_DISSECTOR=1 cargo nextest run -p hopr-transport --features capture --lib dissector
local WIRE = {
  capture = {
    format_version = 1,
    link_type = 147,
    wtap_encap = "USER0",
    frame_type_names = {
      [0x00] = "Final",
      [0x01] = "Forwarded",
      [0x02] = "Outgoing",
      [0x03] = "AcknowledgementIn",
      [0x04] = "AcknowledgementOut",
    },
    frame_type = {
      Final = 0,
      Forwarded = 1,
      Outgoing = 2,
      InAck = 3,
      OutAck = 4,
    },
  },
  size = {
    packet_tag = 16,
    public_key = 32,
    pseudonym = 10,
    half_key = 32,
    ack_challenge = 33,
    acknowledgement = 96,
    surb = 402,
    hopr_packet = 1461,
  },
  ticket = {
    size = 132,
    counterparty = 20,
    amount = 12,
    index = 6,
    channel_epoch = 3,
    win_prob = 7,
    eth_challenge = 20,
    signature = 64,
  },
  app = {
    tag_size = 8,
    payload_size = 1030,
    reserved_upper_bound = 16,
    undefined_tag = 15,
    reserved_tag_names = {
      [0x00] = "Probe",
      [0x01] = "Start",
      [0x02] = "Session",
      [0x0f] = "Undefined",
    },
    reserved_tag = {
      probe = 0,
      start = 1,
      session = 2,
    },
    packet_signals = {
      { bit = 0x01, name = "SurbDistress" },
      { bit = 0x03, name = "OutOfSurbs" },
    },
  },
  probe = {
    version = 1,
    header_size = 2,
    nonce_size = 32,
    telemetry_id_size = 8,
    telemetry_path_size = 40,
    telemetry_timestamp_size = 16,
    message_names = {
      [0x00] = "Telemetry",
      [0x01] = "Probe",
    },
    message = {
      Telemetry = 0,
      Probe = 1,
    },
    neighbor_names = {
      [0x00] = "Ping",
      [0x01] = "Pong",
    },
  },
  start = {
    version = 3,
    header_size = 4,
    challenge_size = 8,
    additional_data_size = 8,
    message_names = {
      [0x00] = "StartSession",
      [0x01] = "SessionEstablished",
      [0x02] = "SsaCommit",
      [0x03] = "SsaRequest",
      [0x04] = "SessionError",
      [0x05] = "KeepAlive",
    },
    message = {
      StartSession = 0,
      SessionEstablished = 1,
      SsaCommit = 2,
      SsaRequest = 3,
      SessionError = 4,
      KeepAlive = 5,
    },
    error_reason_names = {
      [0x00] = "Unknown",
      [0x01] = "No slots available",
      [0x02] = "Busy",
      [0x03] = "Unacceptable PIX parameters",
      [0x04] = "Target not admitted",
    },
    error_identifier_names = {
      [0x00] = "Challenge",
      [0x01] = "SessionId",
    },
    error_identifier_challenge = 0x00,
    capabilities = {
      { bit = 0x08, name = "Segmentation" },
      { bit = 0x0c, name = "RetransmissionAck" },
      { bit = 0x0a, name = "RetransmissionNack" },
      { bit = 0x09, name = "NoDelay" },
      { bit = 0x10, name = "NoRateControl" },
      { bit = 0x20, name = "UsePIX" },
    },
    keep_alive_flags = {
      { bit = 0x01, name = "BalancerTarget" },
      { bit = 0x02, name = "BalancerState" },
    },
  },
  pix = {
    ssa_index_size = 4,
    polynomial_index_size = 2,
    coefficient_index_size = 2,
    missing_run_entry_size = 8,
    max_polys_per_ssa = 16192,
    max_ssas_per_request = 28,
    build_suite = 0,
    suite_names = {
      [0x00] = "BabyJubJub",
      [0x01] = "secp256k1",
    },
    suite = {
      [0x00] = {
        group_repr = 32,
        commitment_proof = 64,
      },
      [0x01] = {
        group_repr = 33,
        commitment_proof = 65,
      },
    },
  },
  session = {
    version = 1,
    header_size = 4,
    segment_header_size = 6,
    frame_id_size = 4,
    seq_num_size = 1,
    seq_terminating_mask = 0x80,
    seq_len_mask = 0x3f,
    request_entry_size = 5,
    max_missing_segments_per_frame = 8,
    ack_entry_size = 4,
    message_names = {
      [0x00] = "Segment",
      [0x01] = "SegmentRequest",
      [0x02] = "FrameAcknowledgements",
    },
    message = {
      Segment = 0,
      SegmentRequest = 1,
      FrameAcknowledgements = 2,
    },
  },
}
-- <<< END GENERATED WIRE CONSTANTS

---------------------------------------------------------------------------------------
-- Shared helpers

-- The Lua BitOp library ships with every Wireshark Lua version.
local band = bit.band
local rshift = bit.rshift
local lshift = bit.lshift

--- Reads a NUL-terminated peer id, adds it to `tree` and returns the offset past the terminator.
local function add_peer_id(tree, field, buffer, offset)
    local text = buffer(offset):stringz()
    tree:add(field, buffer(offset, text:len() + 1), text)
    return offset + text:len() + 1, text
end

--- Returns the list of names in `flags` whose whole mask is present in `value`.
---
--- `== mask` rather than `~= 0` on purpose: several HOPR flag values are composite (for instance
--- `RetransmissionAck` implies `Segmentation`, and `OutOfSurbs` implies `SurbDistress`), so a
--- single-bit test would report a capability the peer never asked for.
local function present_flags(flags, value)
    local out = {}
    for _, flag in ipairs(flags) do
        if band(value, flag.bit) == flag.bit then
            out[#out + 1] = flag.name
        end
    end
    return out
end

--- Adds one filterable item per set flag, plus a summary on the parent item.
local function add_flags(tree, field, flags, value)
    local names = present_flags(flags, value)
    if #names == 0 then
        tree:append_text(" (none)")
        return
    end
    tree:append_text(" (" .. table.concat(names, ", ") .. ")")
    for _, name in ipairs(names) do
        tree:add(field, name)
    end
end

--- Byte length of the CBOR item starting at `offset`, or nil if it is malformed or truncated.
---
--- Needed because several messages interleave CBOR regions with fixed-width fields, so the
--- dissector has to know where a CBOR item *ends* to keep walking. Wireshark's own `cbor` dissector
--- consumes a whole buffer and does not report a length, so it can only be handed a region whose
--- extent is already known.
local function cbor_item_len(buffer, offset)
    local remaining = buffer:len() - offset
    if remaining < 1 then
        return nil
    end

    local initial = buffer(offset, 1):uint()
    local major = rshift(initial, 5)
    local info = band(initial, 0x1f)
    local head = 1
    local argument = info

    if info == 24 then
        head, argument = 2, nil
    elseif info == 25 then
        head, argument = 3, nil
    elseif info == 26 then
        head, argument = 5, nil
    elseif info == 27 then
        head, argument = 9, nil
    elseif info == 31 then
        -- Indefinite length: walk items until the break marker (0xff).
        if major ~= 2 and major ~= 3 and major ~= 4 and major ~= 5 then
            return nil
        end
        local cursor = offset + 1
        while cursor < buffer:len() do
            if buffer(cursor, 1):uint() == 0xff then
                return cursor + 1 - offset
            end
            local inner = cbor_item_len(buffer, cursor)
            if inner == nil then
                return nil
            end
            cursor = cursor + inner
        end
        return nil
    elseif info > 27 then
        return nil
    end

    if argument == nil then
        if remaining < head then
            return nil
        end
        -- `TvbRange:uint()` tops out at four bytes, so the eight-byte form needs `uint64()`.
        local raw = buffer(offset + 1, head - 1)
        argument = head == 9 and raw:uint64():tonumber() or raw:uint()
    end

    if major == 0 or major == 1 or major == 7 then
        return head
    elseif major == 2 or major == 3 then
        -- Byte and text strings: the argument is the payload length.
        if remaining < head + argument then
            return nil
        end
        return head + argument
    elseif major == 6 then
        -- Tag: followed by exactly one tagged item.
        local inner = cbor_item_len(buffer, offset + head)
        return inner and head + inner or nil
    end

    -- Arrays (4) carry `argument` items, maps (5) carry twice that many.
    local items = major == 5 and argument * 2 or argument
    local cursor = offset + head
    for _ = 1, items do
        local inner = cbor_item_len(buffer, cursor)
        if inner == nil then
            return nil
        end
        cursor = cursor + inner
    end
    return cursor - offset
end

--- Hands a CBOR region to Wireshark's `cbor` dissector, falling back to raw bytes.
local cbor_dissector = Dissector.get("cbor")
local function add_cbor(tree, field, label, buffer, offset, length, pinfo)
    if length == nil or length <= 0 or offset + length > buffer:len() then
        if offset < buffer:len() then
            tree:add(field, buffer(offset))
        end
        return buffer:len()
    end
    if cbor_dissector ~= nil then
        cbor_dissector:call(buffer(offset, length):tvb(), pinfo, tree:add(label))
    else
        tree:add(field, buffer(offset, length))
    end
    return offset + length
end

---------------------------------------------------------------------------------------
-- HOPR Probe protocol

local hopr_probe = Proto("hopr_probe", "HOPR Probe Protocol")

local probe_fields = {
    version = ProtoField.uint8("hopr_probe.version", "Version", base.DEC),
    type = ProtoField.uint8("hopr_probe.type", "Type", base.HEX, WIRE.probe.message_names),

    probe_type = ProtoField.uint8("hopr_probe.probe.type", "Probe Type", base.HEX, WIRE.probe.neighbor_names),
    probe_nonce = ProtoField.bytes("hopr_probe.probe.nonce", "Probe Nonce"),

    tele_id = ProtoField.bytes("hopr_probe.telemetry.id", "Telemetry ID"),
    tele_path = ProtoField.bytes("hopr_probe.telemetry.path", "Telemetry Path"),
    tele_ts = ProtoField.bytes("hopr_probe.telemetry.ts", "Telemetry Timestamp"),
}

local probe_experts = {
    version = ProtoExpert.new("hopr_probe.version.unsupported", "Unsupported Probe protocol version",
        expert.group.PROTOCOL, expert.severity.ERROR),
    malformed = ProtoExpert.new("hopr_probe.malformed", "Malformed Probe message",
        expert.group.MALFORMED, expert.severity.ERROR),
}

hopr_probe.fields = probe_fields
hopr_probe.experts = probe_experts

local function dissect_hopr_probe(buffer, pinfo, tree)
    local subtree = tree:add(hopr_probe, buffer())
    local offset = 0

    if buffer:len() < WIRE.probe.header_size then
        subtree:add_proto_expert_info(probe_experts.malformed, "truncated Probe header")
        return buffer:len()
    end

    subtree:add(probe_fields.version, buffer(offset, 1))
    local version = buffer(offset, 1):uint()
    if version ~= WIRE.probe.version then
        subtree:add_proto_expert_info(probe_experts.version,
            "Unsupported Probe version " .. version .. " (expected " .. WIRE.probe.version .. ")")
        return buffer:len()
    end
    offset = offset + 1

    subtree:add(probe_fields.type, buffer(offset, 1))
    local msg_type = buffer(offset, 1):uint()
    offset = offset + 1

    local name = WIRE.probe.message_names[msg_type]
    if name == nil then
        subtree:add_proto_expert_info(probe_experts.malformed, "Unknown Probe message type " .. msg_type)
        return buffer:len()
    end

    if msg_type == WIRE.probe.message.Telemetry then
        pinfo.cols.info:append(", Telemetry")

        local size = WIRE.probe.telemetry_id_size + WIRE.probe.telemetry_path_size
            + WIRE.probe.telemetry_timestamp_size
        if buffer:len() < offset + size then
            subtree:add_proto_expert_info(probe_experts.malformed, "truncated Telemetry message")
            return buffer:len()
        end

        local tele_tree = subtree:add("Telemetry")
        tele_tree:add(probe_fields.tele_id, buffer(offset, WIRE.probe.telemetry_id_size))
        offset = offset + WIRE.probe.telemetry_id_size

        tele_tree:add(probe_fields.tele_path, buffer(offset, WIRE.probe.telemetry_path_size))
        offset = offset + WIRE.probe.telemetry_path_size

        tele_tree:add(probe_fields.tele_ts, buffer(offset, WIRE.probe.telemetry_timestamp_size))
        offset = offset + WIRE.probe.telemetry_timestamp_size
    else
        if buffer:len() < offset + 1 + WIRE.probe.nonce_size then
            subtree:add_proto_expert_info(probe_experts.malformed, "truncated Probe message")
            return buffer:len()
        end

        local probe_tree = subtree:add("Probe")
        probe_tree:add(probe_fields.probe_type, buffer(offset, 1))
        pinfo.cols.info:append(", " .. (WIRE.probe.neighbor_names[buffer(offset, 1):uint()] or "Unknown"))
        offset = offset + 1

        probe_tree:add(probe_fields.probe_nonce, buffer(offset, WIRE.probe.nonce_size))
        offset = offset + WIRE.probe.nonce_size
    end

    return offset
end

---------------------------------------------------------------------------------------
-- HOPR Start protocol

local hopr_start = Proto("hopr_start", "HOPR Start Protocol")

local start_fields = {
    version = ProtoField.uint8("hopr_start.version", "Version", base.DEC),
    type = ProtoField.uint8("hopr_start.type", "Type", base.HEX, WIRE.start.message_names),
    length = ProtoField.uint16("hopr_start.len", "Payload length", base.DEC),
    challenge = ProtoField.uint64("hopr_start.challenge", "Challenge", base.HEX),
    session_id = ProtoField.bytes("hopr_start.session_id", "Session ID (CBOR encoded)"),

    capabilities = ProtoField.uint8("hopr_start.init.capabilities", "Capabilities", base.HEX),
    capability = ProtoField.string("hopr_start.init.capability", "Capability"),
    target = ProtoField.bytes("hopr_start.init.target", "Target (CBOR encoded)"),
    init_ad_data = ProtoField.uint64("hopr_start.init.additional_data", "Additional data", base.HEX),

    ka_flags = ProtoField.uint8("hopr_start.keep_alive.flags", "Flags", base.HEX),
    ka_flag = ProtoField.string("hopr_start.keep_alive.flag", "Flag"),
    ka_additional_data = ProtoField.uint64("hopr_start.keep_alive.additional_data", "Additional data", base.HEX),

    err_identifier = ProtoField.uint8("hopr_start.error.identifier", "Identifies", base.HEX,
        WIRE.start.error_identifier_names),
    err_reason = ProtoField.uint8("hopr_start.error.reason", "Error reason", base.HEX, WIRE.start.error_reason_names),

    ssa_index = ProtoField.uint32("hopr_start.ssa.index", "SSA index", base.DEC),
    ssa_coefficient_index = ProtoField.uint16("hopr_start.ssa.coefficient_index", "Coefficient index", base.DEC),
    ssa_num_polys = ProtoField.uint16("hopr_start.ssa.num_polynomials", "Number of polynomials", base.DEC),
    ssa_proof = ProtoField.bytes("hopr_start.ssa.commitment_proof", "Commitment proof of knowledge"),
    ssa_poly_index = ProtoField.uint16("hopr_start.ssa.polynomial_index", "Polynomial index", base.DEC),
    ssa_commitment = ProtoField.bytes("hopr_start.ssa.commitment", "Coefficient commitment"),

    pix_params = ProtoField.uint32("hopr_start.ssa.params", "PIX parameters", base.HEX),
    pix_suite = ProtoField.uint8("hopr_start.ssa.params.suite", "Curve suite", base.DEC, WIRE.pix.suite_names),
    pix_polys = ProtoField.uint16("hopr_start.ssa.params.polys_per_ssa", "Polynomials per SSA", base.DEC),
    pix_shares = ProtoField.uint8("hopr_start.ssa.params.shares_per_poly", "Shares per polynomial", base.DEC),
    pix_surplus = ProtoField.uint8("hopr_start.ssa.params.surplus_shares", "Surplus shares", base.DEC),
    deposit_data = ProtoField.bytes("hopr_start.ssa.deposit_data", "Deposit data (CBOR encoded)"),
    ssa_num_commitments = ProtoField.uint16("hopr_start.ssa.num_commitments", "Number of commitments", base.DEC),
    ssa_num_missing = ProtoField.uint16("hopr_start.ssa.num_missing_runs", "Number of missing runs", base.DEC),
    ssa_run_first = ProtoField.uint16("hopr_start.ssa.missing_run.first", "First polynomial index", base.DEC),
    ssa_run_last = ProtoField.uint16("hopr_start.ssa.missing_run.last", "Last polynomial index", base.DEC),
}

local start_experts = {
    version = ProtoExpert.new("hopr_start.version.unsupported", "Unsupported Start protocol version",
        expert.group.PROTOCOL, expert.severity.ERROR),
    malformed = ProtoExpert.new("hopr_start.malformed", "Malformed Start message",
        expert.group.MALFORMED, expert.severity.ERROR),
}

hopr_start.fields = start_fields
hopr_start.experts = start_experts

-- Which curve suite the capturing node was built with decides the width of every PIX commitment on
-- the wire. `SsaRequest` announces it in its parameter word, but `SsaCommit` does not, so a capture
-- from a node built for the other curve needs to be told.
local pix_suite_pref_enum = {}
for value, name in pairs(WIRE.pix.suite_names) do
    pix_suite_pref_enum[#pix_suite_pref_enum + 1] = { value + 1, name, value }
end
table.sort(pix_suite_pref_enum, function(a, b) return a[3] < b[3] end)

hopr_start.prefs.pix_suite = Pref.enum("PIX curve suite", WIRE.pix.build_suite,
    "Curve suite the capturing node was built with; decides the width of PIX commitments.",
    pix_suite_pref_enum, false)

local function pix_sizes(suite)
    return WIRE.pix.suite[suite] or WIRE.pix.suite[WIRE.pix.build_suite]
end

local function dissect_start_ssa_commit(buffer, pinfo, subtree, offset, body_end)
    local sizes = pix_sizes(hopr_start.prefs.pix_suite)
    local fixed = WIRE.pix.ssa_index_size + WIRE.pix.coefficient_index_size + WIRE.pix.polynomial_index_size

    if body_end - offset <= fixed then
        subtree:add_proto_expert_info(start_experts.malformed, "truncated SsaCommit message")
        return body_end
    end

    local commit_tree = subtree:add("SSA Client Commitment")
    commit_tree:add(start_fields.ssa_index, buffer(offset, WIRE.pix.ssa_index_size))
    offset = offset + WIRE.pix.ssa_index_size

    commit_tree:add(start_fields.ssa_coefficient_index, buffer(offset, WIRE.pix.coefficient_index_size))
    local coefficient_index = buffer(offset, WIRE.pix.coefficient_index_size):uint()
    offset = offset + WIRE.pix.coefficient_index_size

    commit_tree:add(start_fields.ssa_num_polys, buffer(offset, WIRE.pix.polynomial_index_size))
    local num_polys = buffer(offset, WIRE.pix.polynomial_index_size):uint()
    offset = offset + WIRE.pix.polynomial_index_size

    if num_polys == 0 or num_polys > WIRE.pix.max_polys_per_ssa then
        commit_tree:add_proto_expert_info(start_experts.malformed,
            "polynomial count " .. num_polys .. " out of range")
        return body_end
    end

    -- The proof rides along with the constant-term messages only, and its presence is implied by the
    -- coefficient index rather than by a flag on the wire.
    if coefficient_index == 0 then
        if body_end - offset <= sizes.commitment_proof then
            commit_tree:add_proto_expert_info(start_experts.malformed, "truncated commitment proof")
            return body_end
        end
        commit_tree:add(start_fields.ssa_proof, buffer(offset, sizes.commitment_proof))
        offset = offset + sizes.commitment_proof
    end

    local entry_size = WIRE.pix.polynomial_index_size + sizes.group_repr
    if body_end - offset <= num_polys * entry_size then
        commit_tree:add_proto_expert_info(start_experts.malformed,
            "message cannot hold " .. num_polys .. " commitments of " .. sizes.group_repr .. " bytes")
        return body_end
    end

    local entries = commit_tree:add("Coefficient commitments (" .. num_polys .. ")")
    for _ = 1, num_polys do
        local entry = entries:add("Polynomial " .. buffer(offset, WIRE.pix.polynomial_index_size):uint())
        entry:add(start_fields.ssa_poly_index, buffer(offset, WIRE.pix.polynomial_index_size))
        offset = offset + WIRE.pix.polynomial_index_size
        entry:add(start_fields.ssa_commitment, buffer(offset, sizes.group_repr))
        offset = offset + sizes.group_repr
    end

    pinfo.cols.info:append(" (" .. num_polys .. " commitments)")
    return add_cbor(commit_tree, start_fields.session_id, "Session ID", buffer, offset, body_end - offset, pinfo)
end

local function dissect_start_ssa_request(buffer, pinfo, subtree, offset, body_end)
    if body_end - offset <= 4 + 1 then
        subtree:add_proto_expert_info(start_experts.malformed, "truncated SsaRequest message")
        return body_end
    end

    local req_tree = subtree:add("SSA Server Request")

    -- suite in bits 31..30, polys_per_ssa in 29..16, shares_per_poly in 15..8, surplus in 7..0.
    local params = buffer(offset, 4):uint()
    local params_tree = req_tree:add(start_fields.pix_params, buffer(offset, 4))
    local suite = rshift(params, 30)
    params_tree:add(start_fields.pix_suite, buffer(offset, 1), suite)
    params_tree:add(start_fields.pix_polys, buffer(offset, 2), band(rshift(params, 16), 0x3fff))
    params_tree:add(start_fields.pix_shares, buffer(offset + 2, 1))
    params_tree:add(start_fields.pix_surplus, buffer(offset + 3, 1))
    offset = offset + 4

    -- The parameter word names the suite, so this message needs no preference to be read.
    local sizes = WIRE.pix.suite[suite] or pix_sizes(hopr_start.prefs.pix_suite)

    local deposit_len = cbor_item_len(buffer, offset)
    if deposit_len == nil then
        req_tree:add_proto_expert_info(start_experts.malformed, "malformed deposit data")
        return body_end
    end
    add_cbor(req_tree, start_fields.deposit_data, "Deposit data", buffer, offset, deposit_len, pinfo)
    offset = offset + deposit_len

    if body_end - offset <= 2 then
        req_tree:add_proto_expert_info(start_experts.malformed, "truncated commitment table")
        return body_end
    end
    req_tree:add(start_fields.ssa_num_commitments, buffer(offset, 2))
    local num_commitments = buffer(offset, 2):uint()
    offset = offset + 2

    local entry_size = WIRE.pix.ssa_index_size + sizes.group_repr
    if body_end - offset < num_commitments * entry_size + 2 then
        req_tree:add_proto_expert_info(start_experts.malformed,
            "message cannot hold " .. num_commitments .. " commitments")
        return body_end
    end

    if num_commitments > 0 then
        local entries = req_tree:add("Server commitments (" .. num_commitments .. ")")
        for _ = 1, num_commitments do
            local entry = entries:add("SSA " .. buffer(offset, WIRE.pix.ssa_index_size):uint())
            entry:add(start_fields.ssa_index, buffer(offset, WIRE.pix.ssa_index_size))
            offset = offset + WIRE.pix.ssa_index_size
            entry:add(start_fields.ssa_commitment, buffer(offset, sizes.group_repr))
            offset = offset + sizes.group_repr
        end
    end

    req_tree:add(start_fields.ssa_num_missing, buffer(offset, 2))
    local num_missing = buffer(offset, 2):uint()
    offset = offset + 2

    if body_end - offset < num_missing * WIRE.pix.missing_run_entry_size then
        req_tree:add_proto_expert_info(start_experts.malformed,
            "message cannot hold " .. num_missing .. " retransmission runs")
        return body_end
    end

    if num_missing > 0 then
        local runs = req_tree:add("Missing polynomial runs (" .. num_missing .. ")")
        for _ = 1, num_missing do
            local run = runs:add("SSA " .. buffer(offset, WIRE.pix.ssa_index_size):uint())
            run:add(start_fields.ssa_index, buffer(offset, WIRE.pix.ssa_index_size))
            offset = offset + WIRE.pix.ssa_index_size
            run:add(start_fields.ssa_run_first, buffer(offset, WIRE.pix.polynomial_index_size))
            offset = offset + WIRE.pix.polynomial_index_size
            run:add(start_fields.ssa_run_last, buffer(offset, WIRE.pix.polynomial_index_size))
            offset = offset + WIRE.pix.polynomial_index_size
        end
        pinfo.cols.info:append(" (retransmit " .. num_missing .. " runs)")
    else
        pinfo.cols.info:append(" (" .. num_commitments .. " SSAs)")
    end

    return add_cbor(req_tree, start_fields.session_id, "Session ID", buffer, offset, body_end - offset, pinfo)
end

local function dissect_hopr_start(buffer, pinfo, tree)
    local subtree = tree:add(hopr_start, buffer())
    local offset = 0

    if buffer:len() < WIRE.start.header_size then
        subtree:add_proto_expert_info(start_experts.malformed, "truncated Start header")
        return buffer:len()
    end

    subtree:add(start_fields.version, buffer(offset, 1))
    local version = buffer(offset, 1):uint()
    if version ~= WIRE.start.version then
        subtree:add_proto_expert_info(start_experts.version,
            "Unsupported Start version " .. version .. " (expected " .. WIRE.start.version .. ")")
        return buffer:len()
    end
    offset = offset + 1

    subtree:add(start_fields.type, buffer(offset, 1))
    local msg_type = buffer(offset, 1):uint()
    offset = offset + 1

    local len = buffer(offset, 2):uint()
    subtree:add(start_fields.length, buffer(offset, 2))
    offset = offset + 2

    local body_end = offset + len
    if body_end > buffer:len() then
        subtree:add_proto_expert_info(start_experts.malformed,
            "payload length " .. len .. " exceeds the message")
        return buffer:len()
    end

    local name = WIRE.start.message_names[msg_type]
    if name == nil then
        subtree:add_proto_expert_info(start_experts.malformed, "Unknown Start message type " .. msg_type)
        return body_end
    end
    pinfo.cols.info:append(", " .. name)

    if msg_type == WIRE.start.message.StartSession then
        local fixed = WIRE.start.challenge_size + 1 + WIRE.start.additional_data_size
        if len < fixed then
            subtree:add_proto_expert_info(start_experts.malformed,
                "payload too short for StartSession (" .. len .. " < " .. fixed .. ")")
            return body_end
        end

        local init_tree = subtree:add("Session Initiation")
        init_tree:add(start_fields.challenge, buffer(offset, WIRE.start.challenge_size))
        offset = offset + WIRE.start.challenge_size

        local caps = buffer(offset, 1):uint()
        add_flags(init_tree:add(start_fields.capabilities, buffer(offset, 1)), start_fields.capability,
            WIRE.start.capabilities, caps)
        offset = offset + 1

        init_tree:add(start_fields.init_ad_data, buffer(offset, WIRE.start.additional_data_size))
        offset = offset + WIRE.start.additional_data_size

        offset = add_cbor(init_tree, start_fields.target, "Target", buffer, offset, body_end - offset, pinfo)
    elseif msg_type == WIRE.start.message.SessionEstablished then
        if len <= WIRE.start.challenge_size then
            subtree:add_proto_expert_info(start_experts.malformed,
                "payload too short for SessionEstablished (" .. len .. " <= " .. WIRE.start.challenge_size .. ")")
            return body_end
        end

        local est_tree = subtree:add("Session Established")
        est_tree:add(start_fields.challenge, buffer(offset, WIRE.start.challenge_size))
        offset = offset + WIRE.start.challenge_size

        offset = add_cbor(est_tree, start_fields.session_id, "Session ID", buffer, offset, body_end - offset, pinfo)
    elseif msg_type == WIRE.start.message.SsaCommit then
        offset = dissect_start_ssa_commit(buffer, pinfo, subtree, offset, body_end)
    elseif msg_type == WIRE.start.message.SsaRequest then
        offset = dissect_start_ssa_request(buffer, pinfo, subtree, offset, body_end)
    elseif msg_type == WIRE.start.message.SessionError then
        if len < 2 then
            subtree:add_proto_expert_info(start_experts.malformed, "payload too short for SessionError")
            return body_end
        end

        local err_tree = subtree:add("Session Error")
        err_tree:add(start_fields.err_identifier, buffer(offset, 1))
        local identifier = buffer(offset, 1):uint()
        offset = offset + 1

        -- The reason is always the last byte; what sits between it and the identifier tag is either
        -- the fixed-width challenge or a CBOR session id.
        local reason_at = body_end - 1
        if identifier == WIRE.start.error_identifier_challenge then
            if reason_at - offset < WIRE.start.challenge_size then
                err_tree:add_proto_expert_info(start_experts.malformed, "truncated error challenge")
                return body_end
            end
            err_tree:add(start_fields.challenge, buffer(offset, WIRE.start.challenge_size))
        else
            add_cbor(err_tree, start_fields.session_id, "Session ID", buffer, offset, reason_at - offset, pinfo)
        end
        offset = reason_at

        err_tree:add(start_fields.err_reason, buffer(offset, 1))
        pinfo.cols.info:append(" (" .. (WIRE.start.error_reason_names[buffer(offset, 1):uint()] or "Unknown") .. ")")
        offset = offset + 1
    elseif msg_type == WIRE.start.message.KeepAlive then
        local fixed = 1 + WIRE.start.additional_data_size
        if len <= fixed then
            subtree:add_proto_expert_info(start_experts.malformed,
                "payload too short for KeepAlive (" .. len .. " <= " .. fixed .. ")")
            return body_end
        end

        local ka_tree = subtree:add("Keep-Alive")
        add_flags(ka_tree:add(start_fields.ka_flags, buffer(offset, 1)), start_fields.ka_flag,
            WIRE.start.keep_alive_flags, buffer(offset, 1):uint())
        offset = offset + 1

        ka_tree:add(start_fields.ka_additional_data, buffer(offset, WIRE.start.additional_data_size))
        offset = offset + WIRE.start.additional_data_size

        offset = add_cbor(ka_tree, start_fields.session_id, "Session ID", buffer, offset, body_end - offset, pinfo)
    end

    return math.max(offset, body_end)
end

---------------------------------------------------------------------------------------
-- HOPR Session protocol

local hopr_session = Proto("hopr_session", "HOPR Session Protocol")

local session_fields = {
    version = ProtoField.uint8("hopr_session.version", "Version", base.DEC),
    type = ProtoField.uint8("hopr_session.type", "Type", base.HEX, WIRE.session.message_names),
    len = ProtoField.uint16("hopr_session.len", "Message Length", base.DEC),

    seg_frame_id = ProtoField.uint32("hopr_session.segment.frame_id", "Frame ID", base.DEC),
    seg_idx = ProtoField.uint8("hopr_session.segment.seg_idx", "Segment Index", base.DEC),
    seg_terminating = ProtoField.bool("hopr_session.segment.terminating", "Terminating", 8, nil,
        WIRE.session.seq_terminating_mask),
    seg_seq_len = ProtoField.uint8("hopr_session.segment.seq_len", "Sequence Length", base.DEC, nil,
        WIRE.session.seq_len_mask),
    seg_data = ProtoField.bytes("hopr_session.segment.data", "Data"),

    req_frame_id = ProtoField.uint32("hopr_session.segment_request.frame_id", "Frame ID", base.DEC),
    req_missing = ProtoField.uint8("hopr_session.segment_request.missing_segments", "Missing segments", base.HEX),
    req_missing_seg = ProtoField.uint8("hopr_session.segment_request.missing_segment", "Missing segment", base.DEC),

    ack_frame_id = ProtoField.uint32("hopr_session.frame_ack.frame_id", "Frame ID", base.DEC),
}

local session_experts = {
    version = ProtoExpert.new("hopr_session.version.unsupported", "Unsupported Session protocol version",
        expert.group.PROTOCOL, expert.severity.ERROR),
    malformed = ProtoExpert.new("hopr_session.malformed", "Malformed Session message",
        expert.group.MALFORMED, expert.severity.ERROR),
}

hopr_session.fields = session_fields
hopr_session.experts = session_experts

local function dissect_hopr_session(buffer, pinfo, tree)
    local subtree = tree:add(hopr_session, buffer())
    local offset = 0

    if buffer:len() < WIRE.session.header_size then
        subtree:add_proto_expert_info(session_experts.malformed, "truncated Session header")
        return buffer:len()
    end

    subtree:add(session_fields.version, buffer(offset, 1))
    local version = buffer(offset, 1):uint()
    if version ~= WIRE.session.version then
        subtree:add_proto_expert_info(session_experts.version,
            "Unsupported Session version " .. version .. " (expected " .. WIRE.session.version .. ")")
        return buffer:len()
    end
    offset = offset + 1

    subtree:add(session_fields.type, buffer(offset, 1))
    local msg_type = buffer(offset, 1):uint()
    offset = offset + 1

    subtree:add(session_fields.len, buffer(offset, 2))
    local msg_len = buffer(offset, 2):uint()
    offset = offset + 2

    local body_end = offset + msg_len
    if body_end > buffer:len() then
        subtree:add_proto_expert_info(session_experts.malformed,
            "message length " .. msg_len .. " exceeds the buffer")
        return buffer:len()
    end

    if msg_type == WIRE.session.message.Segment then
        if msg_len < WIRE.session.segment_header_size then
            subtree:add_proto_expert_info(session_experts.malformed, "truncated Segment header")
            return body_end
        end

        local frame_id = buffer(offset, WIRE.session.frame_id_size):uint()
        local seg_idx = buffer(offset + WIRE.session.frame_id_size, 1):uint()
        pinfo.cols.info:append(", Segment (" .. frame_id .. "," .. seg_idx .. ")")

        local seg_tree = subtree:add("Segment")
        seg_tree:add(session_fields.seg_frame_id, buffer(offset, WIRE.session.frame_id_size))
        offset = offset + WIRE.session.frame_id_size
        seg_tree:add(session_fields.seg_idx, buffer(offset, 1))
        offset = offset + 1

        local seg_flags = seg_tree:add("Sequence flags")
        seg_flags:add(session_fields.seg_terminating, buffer(offset, 1))
        seg_flags:add(session_fields.seg_seq_len, buffer(offset, 1))
        if band(buffer(offset, 1):uint(), WIRE.session.seq_terminating_mask) ~= 0 then
            pinfo.cols.info:append(" [F]")
        end
        offset = offset + 1

        local data_len = msg_len - WIRE.session.segment_header_size
        if data_len > 0 then
            local data_buf = buffer(offset, data_len)
            -- Offer the payload to Wireshark's heuristics so tunnelled traffic decodes on its own.
            local data_tvb = data_buf:tvb()
            if not DissectorTable.try_heuristics("udp", data_tvb, pinfo, seg_tree)
                and not DissectorTable.try_heuristics("tcp", data_tvb, pinfo, seg_tree) then
                seg_tree:add(session_fields.seg_data, data_buf)
            end
            offset = offset + data_len
        else
            seg_tree:add("No data")
        end
    elseif msg_type == WIRE.session.message.SegmentRequest then
        -- A fixed-size table zero-padded to the end of the message; frame ID 0 is not a valid entry.
        local count = 0
        while offset + WIRE.session.request_entry_size <= body_end do
            local frame_id = buffer(offset, WIRE.session.frame_id_size):uint()
            if frame_id == 0 then
                break
            end

            local req_tree = subtree:add("SegmentRequest[" .. count .. "]")
            req_tree:add(session_fields.req_frame_id, buffer(offset, WIRE.session.frame_id_size))
            offset = offset + WIRE.session.frame_id_size

            local bitmap = buffer(offset, 1):uint()
            local missing = req_tree:add(session_fields.req_missing, buffer(offset, 1))
            for index = 0, WIRE.session.max_missing_segments_per_frame - 1 do
                -- The bitmap is most-significant-bit first: bit 7 is segment 0.
                if band(bitmap, lshift(1, 7 - index)) ~= 0 then
                    missing:add(session_fields.req_missing_seg, buffer(offset, 1), index)
                end
            end
            offset = offset + 1
            count = count + 1
        end
        pinfo.cols.info:append(", SegmentRequest (" .. count .. ")")
        offset = body_end
    elseif msg_type == WIRE.session.message.FrameAcknowledgements then
        local count = 0
        while offset + WIRE.session.ack_entry_size <= body_end do
            local frame_id = buffer(offset, WIRE.session.ack_entry_size):uint()
            if frame_id == 0 then
                break
            end

            subtree:add(session_fields.ack_frame_id, buffer(offset, WIRE.session.ack_entry_size))
            offset = offset + WIRE.session.ack_entry_size
            count = count + 1
        end
        pinfo.cols.info:append(", FrameAcknowledgements (" .. count .. ")")
        offset = body_end
    else
        subtree:add_proto_expert_info(session_experts.malformed, "Unknown Session message type " .. msg_type)
        offset = body_end
    end

    return math.max(offset, body_end)
end

function hopr_session.dissector(buffer, pinfo, tree)
    if buffer:len() < 1 then
        return 0
    end
    pinfo.cols.protocol = "HOPR Session"
    pinfo.cols.info = "Session"
    return dissect_hopr_session(buffer, pinfo, tree)
end

---------------------------------------------------------------------------------------
-- HOPR capture frames

local hopr_proto = Proto("hopr", "HOPR Protocol")

local hopr_fields = {
    format_version = ProtoField.uint8("hopr.capture_version", "Capture format version", base.DEC),
    type = ProtoField.uint8("hopr.type", "Packet Type", base.DEC, WIRE.capture.frame_type_names),

    packet_tag = ProtoField.bytes("hopr.packet_tag", "Packet Tag"),
    previous_hop = ProtoField.bytes("hopr.previous_hop", "Previous Hop"),
    previous_hop_peer_id = ProtoField.string("hopr.previous_hop_peer_id", "Previous Hop (Peer ID)"),
    next_hop = ProtoField.bytes("hopr.next_hop", "Next Hop"),
    next_hop_peer_id = ProtoField.string("hopr.next_hop_peer_id", "Next Hop (Peer ID)"),
    num_surbs = ProtoField.uint8("hopr.num_surbs", "Number of SURBs", base.DEC),
    is_fwd = ProtoField.bool("hopr.is_forwarded", "Is forwarded"),
    data_len = ProtoField.uint16("hopr.data_len", "Data Length", base.DEC),
    raw_data = ProtoField.bytes("hopr.raw_data", "Raw packet data"),
    signals = ProtoField.uint8("hopr.packet_signals", "Packet signals", base.HEX),
    signal = ProtoField.string("hopr.packet_signal", "Packet signal"),

    ticket_len = ProtoField.uint8("hopr.ticket.len", "Ticket length", base.DEC),
    ticket_counterparty = ProtoField.bytes("hopr.ticket.counterparty", "Counterparty"),
    ticket_amount = ProtoField.bytes("hopr.ticket.amount", "Amount"),
    ticket_index = ProtoField.uint64("hopr.ticket.index", "Index", base.DEC),
    ticket_epoch = ProtoField.uint24("hopr.ticket.channel_epoch", "Channel epoch", base.DEC),
    ticket_challenge = ProtoField.bytes("hopr.ticket.challenge", "Ethereum challenge"),
    ticket_luck = ProtoField.bytes("hopr.ticket.luck", "Encoded winning probability"),
    ticket_win_prob = ProtoField.double("hopr.ticket.win_prob", "Winning probability"),
    ticket_signature = ProtoField.bytes("hopr.ticket.signature", "Signature"),

    sender_pseudonym = ProtoField.bytes("hopr.sender_pseudonym", "Sender Pseudonym"),
    ack_key = ProtoField.bytes("hopr.ack.key", "ACK Key"),
    ack_sig = ProtoField.bytes("hopr.ack.sig", "ACK Signature"),
    ack_count = ProtoField.uint16("hopr.ack.count", "Number of acknowledgements", base.DEC),
    ack_random = ProtoField.bool("hopr.ack.is_random", "Is random"),
    challenge = ProtoField.bytes("hopr.challenge", "Acknowledgement challenge"),

    appdata_tag = ProtoField.uint64("hopr.appdata.tag", "Tag", base.DEC),
    appdata_type = ProtoField.string("hopr.appdata.type", "Type"),
    appdata_data = ProtoField.bytes("hopr.appdata.data", "Data"),
}

local hopr_experts = {
    version = ProtoExpert.new("hopr.capture_version.unsupported", "Unsupported capture format version",
        expert.group.PROTOCOL, expert.severity.ERROR),
    malformed = ProtoExpert.new("hopr.malformed", "Malformed capture frame",
        expert.group.MALFORMED, expert.severity.ERROR),
    trailing = ProtoExpert.new("hopr.trailing_bytes", "Undissected trailing bytes in the capture frame",
        expert.group.UNDECODED, expert.severity.WARN),
    random_ack = ProtoExpert.new("hopr.ack.random", "Acknowledgement is random due to a processing error",
        expert.group.PROTOCOL, expert.severity.NOTE),
}

hopr_proto.fields = hopr_fields
hopr_proto.experts = hopr_experts

--- Decodes the encoded winning probability into the double the node would compute.
---
--- The encoding is the mantissa of an IEEE-754 double in [1,2) minus one, so the value is simply the
--- big-endian integer plus one over 2^(8 * width). Lua numbers are doubles, so the last few bits of
--- a 56-bit encoding are lost here exactly as they are on the Rust side, which packs the same
--- integer into a 52-bit mantissa; the two agree to within an ulp.
local function luck_to_double(range)
    local all_zeros, all_ff = true, true
    for i = 0, WIRE.ticket.win_prob - 1 do
        local byte = range(i, 1):uint()
        if byte ~= 0x00 then all_zeros = false end
        if byte ~= 0xff then all_ff = false end
    end
    -- All zeros means "never", which the formula below would render as 2^-56 rather than 0.
    if all_zeros then return 0.0 end
    if all_ff then return 1.0 end

    local value = 0.0
    for i = 0, WIRE.ticket.win_prob - 1 do
        value = value * 256.0 + range(i, 1):uint()
    end
    return (value + 1.0) / (2.0 ^ (8 * WIRE.ticket.win_prob))
end

local function dissect_ticket(buffer, tree, offset)
    local ticket_tree = tree:add("Ticket")

    local ticket_len = buffer(offset, 1):uint()
    ticket_tree:add(hopr_fields.ticket_len, buffer(offset, 1))
    offset = offset + 1

    if ticket_len == 0 then
        ticket_tree:append_text(" (none)")
        return offset
    end
    if ticket_len ~= WIRE.ticket.size then
        ticket_tree:add_proto_expert_info(hopr_experts.malformed,
            "Invalid ticket length " .. ticket_len .. " (expected " .. WIRE.ticket.size .. ")")
        return offset + ticket_len
    end

    ticket_tree:add(hopr_fields.ticket_counterparty, buffer(offset, WIRE.ticket.counterparty))
    offset = offset + WIRE.ticket.counterparty

    ticket_tree:add(hopr_fields.ticket_amount, buffer(offset, WIRE.ticket.amount))
    offset = offset + WIRE.ticket.amount

    ticket_tree:add(hopr_fields.ticket_index, buffer(offset, WIRE.ticket.index))
    offset = offset + WIRE.ticket.index

    ticket_tree:add(hopr_fields.ticket_epoch, buffer(offset, WIRE.ticket.channel_epoch))
    offset = offset + WIRE.ticket.channel_epoch

    ticket_tree:add(hopr_fields.ticket_luck, buffer(offset, WIRE.ticket.win_prob))
    ticket_tree:add(hopr_fields.ticket_win_prob, buffer(offset, WIRE.ticket.win_prob),
        luck_to_double(buffer(offset, WIRE.ticket.win_prob)))
    offset = offset + WIRE.ticket.win_prob

    ticket_tree:add(hopr_fields.ticket_challenge, buffer(offset, WIRE.ticket.eth_challenge))
    offset = offset + WIRE.ticket.eth_challenge

    ticket_tree:add(hopr_fields.ticket_signature, buffer(offset, WIRE.ticket.signature))
    offset = offset + WIRE.ticket.signature

    return offset
end

--- Dissects an `ApplicationData` (8-byte tag followed by the payload) and dispatches on the tag.
local function dissect_appdata(buffer, tree, offset, data_len, pinfo)
    local appdata_tree = tree:add("ApplicationData")

    if data_len < WIRE.app.tag_size then
        appdata_tree:add_proto_expert_info(hopr_experts.malformed, "payload is shorter than the application tag")
        return offset + data_len
    end

    local tag = buffer(offset, WIRE.app.tag_size):uint64():tonumber()
    appdata_tree:add(hopr_fields.appdata_tag, buffer(offset, WIRE.app.tag_size))
    offset = offset + WIRE.app.tag_size

    local payload_len = data_len - WIRE.app.tag_size
    if payload_len == 0 then
        appdata_tree:add(hopr_fields.appdata_type, "Empty")
        return offset
    end

    local payload = buffer(offset, payload_len):tvb()
    if tag == WIRE.app.reserved_tag.probe then
        appdata_tree:add(hopr_fields.appdata_type, WIRE.app.reserved_tag_names[tag])
        dissect_hopr_probe(payload, pinfo, appdata_tree)
    elseif tag == WIRE.app.reserved_tag.start then
        appdata_tree:add(hopr_fields.appdata_type, WIRE.app.reserved_tag_names[tag])
        dissect_hopr_start(payload, pinfo, appdata_tree)
    elseif tag == WIRE.app.reserved_tag.session then
        appdata_tree:add(hopr_fields.appdata_type, WIRE.app.reserved_tag_names[tag])
        dissect_hopr_session(payload, pinfo, appdata_tree)
    else
        -- Everything else is either an unassigned reserved tag or ordinary application traffic.
        local name = WIRE.app.reserved_tag_names[tag]
        if name == nil then
            name = tag < WIRE.app.reserved_upper_bound and WIRE.app.reserved_tag_names[WIRE.app.undefined_tag]
                or "Application"
        end
        appdata_tree:add(hopr_fields.appdata_type, name)
        appdata_tree:add(hopr_fields.appdata_data, buffer(offset, payload_len))
        pinfo.cols.info:append(", " .. name)
    end

    return offset + payload_len
end

--- Reads the `me`/`next_hop` (or `previous_hop`/`me`) key pair that opens most frames.
local function dissect_hops(tree, buffer, offset, pinfo, src_field, src_id_field, dst_field, dst_id_field)
    tree:add(src_field, buffer(offset, WIRE.size.public_key))
    offset = offset + WIRE.size.public_key
    local src
    offset, src = add_peer_id(tree, src_id_field, buffer, offset)
    pinfo.cols.src = src

    tree:add(dst_field, buffer(offset, WIRE.size.public_key))
    offset = offset + WIRE.size.public_key
    local dst
    offset, dst = add_peer_id(tree, dst_id_field, buffer, offset)
    pinfo.cols.dst = dst

    return offset
end

--- Reads the `u16`-prefixed acknowledgement batch shared by both acknowledgement frames.
local function dissect_acks(tree, buffer, offset)
    local count = buffer(offset, 2):uint()
    local ack_tree = tree:add("Acknowledgements (" .. count .. ")")
    ack_tree:add(hopr_fields.ack_count, buffer(offset, 2))
    offset = offset + 2

    for index = 0, count - 1 do
        if offset + WIRE.size.acknowledgement > buffer:len() then
            ack_tree:add_proto_expert_info(hopr_experts.malformed, "acknowledgement batch is truncated")
            return buffer:len()
        end
        local entry = ack_tree:add("Acknowledgement[" .. index .. "]")
        entry:add(hopr_fields.ack_key, buffer(offset, WIRE.size.half_key))
        offset = offset + WIRE.size.half_key
        entry:add(hopr_fields.ack_sig, buffer(offset, WIRE.size.acknowledgement - WIRE.size.half_key))
        offset = offset + WIRE.size.acknowledgement - WIRE.size.half_key
    end

    return offset
end

function hopr_proto.dissector(buffer, pinfo, tree)
    local length = buffer:len()
    if length < 2 then
        return 0
    end

    pinfo.cols.protocol = "HOPR"
    local subtree = tree:add(hopr_proto, buffer(), "HOPR Protocol")
    local offset = 0

    subtree:add(hopr_fields.format_version, buffer(offset, 1))
    local format_version = buffer(offset, 1):uint()
    if format_version ~= WIRE.capture.format_version then
        subtree:add_proto_expert_info(hopr_experts.version,
            "Capture written by a node using format version " .. format_version .. "; this dissector reads version "
            .. WIRE.capture.format_version .. ". Update hopr.lua from the matching hoprnet revision.")
        return length
    end
    offset = offset + 1

    local pkt_type = buffer(offset, 1):uint()
    subtree:add(hopr_fields.type, buffer(offset, 1))
    offset = offset + 1

    if pkt_type == WIRE.capture.frame_type.Final then
        pinfo.cols.info:set("Incoming")

        local final_tree = subtree:add("FinalPacket")
        final_tree:add(hopr_fields.packet_tag, buffer(offset, WIRE.size.packet_tag))
        offset = offset + WIRE.size.packet_tag

        offset = dissect_hops(final_tree, buffer, offset, pinfo, hopr_fields.previous_hop,
            hopr_fields.previous_hop_peer_id, hopr_fields.next_hop, hopr_fields.next_hop_peer_id)

        final_tree:add(hopr_fields.sender_pseudonym, buffer(offset, WIRE.size.pseudonym))
        offset = offset + WIRE.size.pseudonym

        final_tree:add(hopr_fields.ack_key, buffer(offset, WIRE.size.half_key))
        offset = offset + WIRE.size.half_key

        add_flags(final_tree:add(hopr_fields.signals, buffer(offset, 1)), hopr_fields.signal,
            WIRE.app.packet_signals, buffer(offset, 1):uint())
        offset = offset + 1

        local data_len = buffer(offset, 2):uint()
        final_tree:add(hopr_fields.data_len, buffer(offset, 2))
        offset = offset + 2

        offset = dissect_appdata(buffer, final_tree, offset, data_len, pinfo)
    elseif pkt_type == WIRE.capture.frame_type.Forwarded then
        pinfo.cols.info:set("Relayed")

        local fwd_tree = subtree:add("ForwardedPacket")
        fwd_tree:add(hopr_fields.packet_tag, buffer(offset, WIRE.size.packet_tag))
        offset = offset + WIRE.size.packet_tag

        offset = dissect_hops(fwd_tree, buffer, offset, pinfo, hopr_fields.previous_hop,
            hopr_fields.previous_hop_peer_id, hopr_fields.next_hop, hopr_fields.next_hop_peer_id)

        fwd_tree:add(hopr_fields.ack_key, buffer(offset, WIRE.size.half_key))
        offset = offset + WIRE.size.half_key

        offset = dissect_ticket(buffer, fwd_tree, offset)

        local data_len = buffer(offset, 2):uint()
        fwd_tree:add(hopr_fields.data_len, buffer(offset, 2))
        offset = offset + 2

        -- Still onion-encrypted for the next hop, so there is nothing further to decode here.
        fwd_tree:add(hopr_fields.raw_data, buffer(offset, data_len))
        offset = offset + data_len
    elseif pkt_type == WIRE.capture.frame_type.Outgoing then
        pinfo.cols.info:set("Outgoing")

        local out_tree = subtree:add("OutgoingPacket")

        offset = dissect_hops(out_tree, buffer, offset, pinfo, hopr_fields.previous_hop,
            hopr_fields.previous_hop_peer_id, hopr_fields.next_hop, hopr_fields.next_hop_peer_id)

        out_tree:add(hopr_fields.challenge, buffer(offset, WIRE.size.ack_challenge))
        offset = offset + WIRE.size.ack_challenge

        offset = dissect_ticket(buffer, out_tree, offset)

        local num_surbs = buffer(offset, 1)
        offset = offset + 1

        local is_fwd = buffer(offset, 1):uint() == 1
        out_tree:add(hopr_fields.is_fwd, buffer(offset, 1))
        offset = offset + 1

        -- A relayed packet is not ours to count SURBs for, and the field is always zero there.
        if not is_fwd then
            out_tree:add(hopr_fields.num_surbs, num_surbs)
        end

        add_flags(out_tree:add(hopr_fields.signals, buffer(offset, 1)), hopr_fields.signal,
            WIRE.app.packet_signals, buffer(offset, 1):uint())
        offset = offset + 1

        local data_len = buffer(offset, 2):uint()
        out_tree:add(hopr_fields.data_len, buffer(offset, 2))
        offset = offset + 2

        if is_fwd then
            out_tree:add(hopr_fields.raw_data, buffer(offset, data_len))
            offset = offset + data_len
        else
            offset = dissect_appdata(buffer, out_tree, offset, data_len, pinfo)
        end
    elseif pkt_type == WIRE.capture.frame_type.InAck then
        pinfo.cols.info:set("Incoming, Acknowledgement")

        local ack_in_tree = subtree:add("Acknowledgement")
        ack_in_tree:add(hopr_fields.packet_tag, buffer(offset, WIRE.size.packet_tag))
        offset = offset + WIRE.size.packet_tag

        offset = dissect_hops(ack_in_tree, buffer, offset, pinfo, hopr_fields.previous_hop,
            hopr_fields.previous_hop_peer_id, hopr_fields.next_hop, hopr_fields.next_hop_peer_id)

        offset = dissect_acks(ack_in_tree, buffer, offset)
    elseif pkt_type == WIRE.capture.frame_type.OutAck then
        pinfo.cols.info:set("Outgoing, Acknowledgement")

        local ack_out_tree = subtree:add("Acknowledgement")

        offset = dissect_hops(ack_out_tree, buffer, offset, pinfo, hopr_fields.previous_hop,
            hopr_fields.previous_hop_peer_id, hopr_fields.next_hop, hopr_fields.next_hop_peer_id)

        local is_random = buffer(offset, 1):uint() == 1
        ack_out_tree:add(hopr_fields.ack_random, buffer(offset, 1))
        if is_random then
            ack_out_tree:add_proto_expert_info(hopr_experts.random_ack)
        end
        offset = offset + 1

        offset = dissect_acks(ack_out_tree, buffer, offset)
    else
        subtree:add_proto_expert_info(hopr_experts.malformed, "Unknown capture frame type " .. pkt_type)
        return length
    end

    -- Every field of a capture frame is accounted for by design; anything left over means this
    -- dissector and the writing node disagree about the layout.
    if offset < length then
        subtree:add_proto_expert_info(hopr_experts.trailing,
            (length - offset) .. " undissected trailing bytes; hopr.lua may be out of date")
    end

    return length
end

---------------------------------------------------------------------------------------
-- Registration

-- Capture files carry a user-defined link type, which is what lets them dissect without the reader
-- having to reach for "Decode As". The `wtap_encap` table is keyed by Wireshark's own encapsulation
-- constant rather than by the LINKTYPE number stored in the file, so it is looked up by name.
local wtap_encap = wtap[WIRE.capture.wtap_encap]
if wtap_encap == nil then
    error("this Wireshark build does not know the link type WTAP_ENCAP_" .. WIRE.capture.wtap_encap)
end
DissectorTable.get("wtap_encap"):add(wtap_encap, hopr_proto)

-- Convenience registrations for replaying frames over a synthetic link.
local ethertype_table = DissectorTable.get("ethertype")
ethertype_table:add(0x1234, hopr_proto)
ethertype_table:add(0x1235, hopr_session)

DissectorTable.get("udp.port"):add(10000, hopr_session)
