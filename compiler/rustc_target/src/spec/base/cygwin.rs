use std::borrow::Cow;

use crate::spec::{
    Cc, DebuginfoKind, LinkerFlavor, Lld, SplitDebuginfo, Target, TargetOptions, cvs,
};

#[derive(Clone, Copy)]
pub(crate) enum CygwinVariant {
    Cygwin,
    MSYS2,
}

fn opts(env: CygwinVariant) -> TargetOptions {
    let mut pre_link_args = TargetOptions::link_args(LinkerFlavor::Gnu(Cc::No, Lld::No), &[
        // FIXME: Disable ASLR for now to fix relocation error
        "--disable-dynamicbase",
        "--enable-auto-image-base",
    ]);
    crate::spec::add_link_args(&mut pre_link_args, LinkerFlavor::Gnu(Cc::Yes, Lld::No), &[
        // Tell GCC to avoid linker plugins, because we are not bundling
        // them with Windows installer, and Rust does its own LTO anyways.
        "-fno-use-linker-plugin",
        "-Wl,--disable-dynamicbase",
        "-Wl,--enable-auto-image-base",
    ]);
    // cygwin runtime lib
    let clib = match env {
        CygwinVariant::Cygwin => "-lcygwin",
        CygwinVariant::MSYS2 => "-lmsys-2.0",
    };
    let cygwin_libs = &[clib, "-lgcc", clib, "-luser32", "-lkernel32", "-lgcc_s"];
    let mut late_link_args =
        TargetOptions::link_args(LinkerFlavor::Gnu(Cc::No, Lld::No), cygwin_libs);
    crate::spec::add_link_args(
        &mut late_link_args,
        LinkerFlavor::Gnu(Cc::Yes, Lld::No),
        cygwin_libs,
    );
    let target_env = match env {
        CygwinVariant::Cygwin => "",
        CygwinVariant::MSYS2 => "msys2",
    };
    TargetOptions {
        os: "cygwin".into(),
        vendor: "pc".into(),
        env: target_env.into(),
        // FIXME(#13846) this should be enabled for cygwin
        function_sections: false,
        linker: Some("gcc".into()),
        dynamic_linking: true,
        dll_prefix: "".into(),
        dll_suffix: ".dll".into(),
        exe_suffix: ".exe".into(),
        families: cvs!["unix"],
        is_like_windows: true,
        allows_weak_linkage: false,
        pre_link_args,
        late_link_args,
        abi_return_struct_as_int: true,
        emit_debug_gdb_scripts: false,
        requires_uwtable: true,
        eh_frame_header: false,
        // FIXME(davidtwco): Support Split DWARF on Cygwin - may require LLVM changes to
        // output DWO, despite using DWARF, doesn't use ELF..
        debuginfo_kind: DebuginfoKind::Pdb,
        supported_split_debuginfo: Cow::Borrowed(&[SplitDebuginfo::Off]),
        ..Default::default()
    }
}

pub(crate) fn x86_64_target(env: CygwinVariant) -> Target {
    let mut base = opts(env);
    base.cpu = "x86-64".into();
    // FIXME: Disable ASLR for now to fix relocation error
    base.add_pre_link_args(LinkerFlavor::Gnu(Cc::No, Lld::No), &[
        "-m",
        "i386pep",
        "--disable-high-entropy-va",
    ]);
    base.add_pre_link_args(LinkerFlavor::Gnu(Cc::Yes, Lld::No), &[
        "-m64",
        "-Wl,--disable-high-entropy-va",
    ]);
    base.max_atomic_width = Some(64);
    let linker = match env {
        CygwinVariant::Cygwin => "x86_64-pc-cygwin-gcc",
        CygwinVariant::MSYS2 => "x86_64-pc-msys-gcc",
    };
    let description = match env {
        CygwinVariant::Cygwin => "64-bit x86 Cygwin",
        CygwinVariant::MSYS2 => "64-bit x86 MSYS2",
    };
    base.linker = Some(linker.into());
    Target {
        llvm_target: "x86_64-pc-cygwin".into(),
        pointer_width: 64,
        data_layout:
            "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128".into(),
        arch: "x86_64".into(),
        options: base,
        metadata: crate::spec::TargetMetadata {
            description: Some(description.into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(true),
        },
    }
}
