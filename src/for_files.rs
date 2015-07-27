/*
    Copyright © 2015 Zetok Zalbavar <zetok@openmailbox.org>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/



/*
    For getting stuff from files
*/
use std::io::prelude::*;
use std::fs::File;


/**
    Function to load save file from `save.tox` file.

    In case where it can't be opened, return an error, so that it could
    be printed, and Tox instance could be initialized without it.
*/
pub fn load_save(f: &str) -> Result<Vec<u8>, String> {
    match File::open(f) {
        Ok(mut file) => {
            let mut res: Vec<u8> = Vec::new();
            drop(file.read_to_end(&mut res));
            Ok(res)
        },

        Err(e) => {
            Err(format!("{}", e))
        },
    }
}
/**
    Function to write save file to storage.

    In case where it can't be written to, return an error, so that it could
    be printed.
*/
pub fn write_save(f: &str, data: Vec<u8>) -> Result<(), String> {
    match File::create(f) {
        Ok(mut file) => {
            drop(file.set_len(data.len() as u64));
            drop(file.write(&data));
            Ok(())
        },

        Err(e) => {
            Err(format!("{}", e))
        },
    }
}
