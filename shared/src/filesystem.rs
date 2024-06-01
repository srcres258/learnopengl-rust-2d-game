// SPDX-License-Identifier: Apache-2.0

// Copyright 2024 src_resources
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;

pub fn get_path(path: String) -> String {
    let mut cur_path = env::current_dir().expect("Failed to obtain current directory.");
    let path_parts: Vec<&str> = path.as_str().split('/').collect();
    for part in path_parts.iter() {
        cur_path.push(part);
    }
    cur_path.into_os_string().into_string().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const FILE: &str = "resources/textures/awesomeface.png";
    const DIR: &str = "resources/textures";

    #[test]
    fn get_path_test_file() {
        let file_name = get_path(FILE.to_string());
        assert!(file_name.len() > 0, "{} failed to be converted into a path.", FILE);
        let file_path = Path::new(&file_name);
        assert!(file_path.exists(), "{} must exist.", FILE);
        assert!(file_path.is_file(), "{} must mean a file.", FILE);
        assert!(!file_path.is_dir(), "{} mustn't mean a directory.", FILE);
    }

    #[test]
    fn get_path_test_dir() {
        let file_name = get_path(DIR.to_string());
        assert!(file_name.len() > 0, "{} failed to be converted into a path.", FILE);
        let file_path = Path::new(&file_name);
        assert!(file_path.exists(), "{} must exist.", DIR);
        assert!(file_path.is_dir(), "{} must mean a directory.", FILE);
        assert!(!file_path.is_file(), "{} mustn't mean a file.", DIR);
    }
}