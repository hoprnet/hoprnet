// Copyright 2019-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Returns `true` if the given string is a proper Rust identifier.
pub fn is_rust_identifier(s: &str) -> bool {
    // Only ascii encoding is allowed.
    // Note: Maybe this check is superseded by the `head` and `tail` check.
    if !s.is_ascii() {
        return false;
    }
    // Trim valid raw identifier prefix
    let trimmed = s.trim_start_matches("r#");
    if let Some((&head, tail)) = trimmed.as_bytes().split_first() {
        // Check if head and tail make up a proper Rust identifier.
        let head_ok = head == b'_' || head.is_ascii_lowercase() || head.is_ascii_uppercase();
        let tail_ok = tail.iter().all(|&ch| {
            ch == b'_' || ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch.is_ascii_digit()
        });
        head_ok && tail_ok
    } else {
        // String is empty and thus not a valid Rust identifier.
        false
    }
}
