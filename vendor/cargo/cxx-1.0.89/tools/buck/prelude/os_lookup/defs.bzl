# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

load("@prelude//:attributes.bzl", "Platform")

OsLookup = provider(fields = ["platform"])

def _os_lookup_impl(ctx: "context"):
    return [DefaultInfo(), OsLookup(platform = ctx.attrs.platform)]

os_lookup = rule(impl = _os_lookup_impl, attrs = {
    "platform": attrs.enum(Platform),
})
