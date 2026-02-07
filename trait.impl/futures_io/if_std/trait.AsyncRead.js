(function() {
    const implementors = Object.fromEntries([["hopr_network_types",[["impl&lt;R, W&gt; AsyncRead for <a class=\"struct\" href=\"hopr_network_types/utils/struct.DuplexIO.html\" title=\"struct hopr_network_types::utils::DuplexIO\">DuplexIO</a>&lt;W, R&gt;<div class=\"where\">where\n    R: AsyncRead,\n    W: AsyncWrite,</div>",0]]],["hopr_protocol_session",[["impl&lt;const C: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, S: <a class=\"trait\" href=\"hopr_protocol_session/trait.SocketState.html\" title=\"trait hopr_protocol_session::SocketState\">SocketState</a>&lt;C&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + 'static&gt; AsyncRead for <a class=\"struct\" href=\"hopr_protocol_session/struct.SessionSocket.html\" title=\"struct hopr_protocol_session::SessionSocket\">SessionSocket</a>&lt;C, S&gt;",0]]],["hopr_transport_session",[["impl AsyncRead for <a class=\"struct\" href=\"hopr_transport_session/struct.HoprSession.html\" title=\"struct hopr_transport_session::HoprSession\">HoprSession</a>",0]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":59,"fragment_lengths":[285,631,199]}