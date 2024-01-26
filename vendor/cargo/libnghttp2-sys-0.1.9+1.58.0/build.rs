extern crate cc;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let lib_version = env::var("CARGO_PKG_VERSION")
        .unwrap()
        .split('+')
        .nth(1)
        .unwrap()
        .to_string();
    let major = lib_version
        .split('.')
        .nth(0)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let minor = lib_version
        .split('.')
        .nth(1)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let patch = lib_version
        .split('.')
        .nth(2)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let ver = fs::read_to_string("nghttp2/lib/includes/nghttp2/nghttp2ver.h.in")
        .unwrap()
        .replace("@PACKAGE_VERSION@", &lib_version)
        .replace(
            "@PACKAGE_VERSION_NUM@",
            &format!("0x{:02x}{:02x}{:02x}", major, minor, patch),
        );

    let install = out_dir.join("i");

    let include = install.join("include");
    let lib = install.join("lib");
    let pkgconfig = lib.join("pkgconfig");
    fs::create_dir_all(include.join("nghttp2")).unwrap();
    fs::create_dir_all(&pkgconfig).unwrap();
    fs::write(include.join("nghttp2/nghttp2ver.h"), ver).unwrap();

    let mut cfg = cc::Build::new();
    cfg.include("nghttp2/lib/includes")
        .include(&include)
        .file("nghttp2/lib/sfparse.c")
        .file("nghttp2/lib/nghttp2_buf.c")
        .file("nghttp2/lib/nghttp2_callbacks.c")
        .file("nghttp2/lib/nghttp2_debug.c")
        .file("nghttp2/lib/nghttp2_extpri.c")
        .file("nghttp2/lib/nghttp2_frame.c")
        .file("nghttp2/lib/nghttp2_hd.c")
        .file("nghttp2/lib/nghttp2_hd_huffman.c")
        .file("nghttp2/lib/nghttp2_hd_huffman_data.c")
        .file("nghttp2/lib/nghttp2_helper.c")
        .file("nghttp2/lib/nghttp2_http.c")
        .file("nghttp2/lib/nghttp2_map.c")
        .file("nghttp2/lib/nghttp2_mem.c")
        .file("nghttp2/lib/nghttp2_npn.c")
        .file("nghttp2/lib/nghttp2_option.c")
        .file("nghttp2/lib/nghttp2_outbound_item.c")
        .file("nghttp2/lib/nghttp2_pq.c")
        .file("nghttp2/lib/nghttp2_priority_spec.c")
        .file("nghttp2/lib/nghttp2_queue.c")
        .file("nghttp2/lib/nghttp2_rcbuf.c")
        .file("nghttp2/lib/nghttp2_session.c")
        .file("nghttp2/lib/nghttp2_stream.c")
        .file("nghttp2/lib/nghttp2_submit.c")
        .file("nghttp2/lib/nghttp2_version.c")
        .file("nghttp2/lib/nghttp2_ratelim.c")
        .file("nghttp2/lib/nghttp2_time.c")
        .warnings(false)
        .define("NGHTTP2_STATICLIB", None)
        .define("HAVE_NETINET_IN", None)
        .define("HAVE_TIME_H", None)
        .out_dir(&lib);

    if target.contains("windows") {
        // Apparently MSVC doesn't have `ssize_t` defined as a type
        if target.contains("msvc") {
            match &env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap()[..] {
                "64" => {
                    cfg.define("ssize_t", "int64_t");
                }
                "32" => {
                    cfg.define("ssize_t", "int32_t");
                }
                s => panic!("unknown pointer size: {}", s),
            }
        }
    } else {
        cfg.define("HAVE_ARPA_INET_H", None);
    }
    cfg.compile("nghttp2");

    println!("cargo:root={}", install.display());

    let pc = fs::read_to_string("nghttp2/lib/libnghttp2.pc.in")
        .unwrap()
        .replace("@prefix@", install.to_str().unwrap())
        .replace("@exec_prefix@", "")
        .replace("@libdir@", lib.to_str().unwrap())
        .replace("@includedir@", include.to_str().unwrap())
        .replace("@VERSION@", &lib_version);
    fs::write(pkgconfig.join("libnghttp2.pc"), pc).unwrap();
    fs::copy(
        "nghttp2/lib/includes/nghttp2/nghttp2.h",
        include.join("nghttp2/nghttp2.h"),
    )
    .unwrap();
}
