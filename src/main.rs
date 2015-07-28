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
    Binding to toxcore
*/
extern crate rstox;
use rstox::core::*;


/*
    For various stuff
*/
extern crate rand;
use rand::ThreadRng;
use rand::Rng;


/*
    For loading and writing Tox data
*/
extern crate chrono;
use chrono::UTC;

mod for_files;


/*
    Bot's own stuff
*/
// TODO: when other functions will be moved from main.rs, things should be
//       added here
mod bootstrap;



struct Bot {
    /**
        Cached RNG, apparently it helps with RNG's performance when it's used
        a lot.
    */
    random: ThreadRng,

    /**
        List of peers to which spam should be sent
    */
    spam: Vec<u32>,

    /**
        Time since last save.
    */
    last_save: i64,
}


/**
    Get some random data as UTF-8 string of `len` length and a source of
    randomness.
*/
unsafe fn rand_string(len: usize, rng: &mut ThreadRng) -> String {
    let mut vec: Vec<u8> = Vec::with_capacity(len);

    for _ in 0..len {
        vec.push(rng.gen::<u8>());
    }

    String::from_utf8_unchecked(vec)
}



/*
    Function to deal with incoming friend requests, all are accepted.
*/
fn on_friend_request(tox: &mut Tox, fpk: PublicKey, msg: String) {
    drop(tox.add_friend_norequest(&fpk));
    println!("\nFriend {} with friend message {:?} was added.", fpk, msg);
}

/*
    Function to deal with friend messages.

    Every message should be replied with a random message.

    Message can trigger constant stream of random messages, turn it off,
    or send an ID to friend.

*/
fn on_friend_message(tox: &mut Tox, fnum: u32, msg: String, bot: &mut Bot) {
    let reply_msg: String = unsafe {
        rand_string((1372.0 * bot.random.gen::<f64>()) as usize, &mut bot.random)
    };

    match &*msg {
        "id" | "ID" => {
            let message = format!("My ID: {}", tox.get_address());
            drop(tox.send_friend_message(fnum, MessageType::Normal, &message));
        },

        "start" => {
            bot.spam.push(fnum);
            println!("Friend {} added to spam list.", fnum);
            println!("Spam list has {} friend(s): {:?}", bot.spam.len(),
                                                         bot.spam);
        },

        "stop" => {
            for f in 0..bot.spam.len() {
                if bot.spam[f] == fnum {
                    drop(bot.spam.remove(f));
                    println!("Friend {} removed from spam list.", fnum);
                    println!("Spam list has {} friend(s): {:?}", bot.spam.len(),
                                                                 bot.spam);
                }
            }
        },

        _ => drop(tox.send_friend_message(fnum, MessageType::Normal,
                &reply_msg)),
    }
}


fn main() {
    /*
        Try to load data file, if not possible, print an error and generate
        new Tox instance.
    */
    let data = match for_files::load_save("bot.tox") {
        Ok(d) => {
            println!("\nSave data loaded.\n");
            Some(d)
        },
        Err(e) => {
            println!("\nError loading save: {}\n", e);
            None
        },
    };
    let mut tox = Tox::new(ToxOptions::new(), data.as_ref()
                                            .map(|x| &**x)).unwrap();


    drop(tox.set_name("THIS. IS. TOX!"));
    drop(tox.set_status_message("Send \"start\" to start spam and \"stop\" to stop it."));

    /*
        Bot stuff
    */
    let mut bot = Bot {
        random: rand::thread_rng(),
        spam: vec![],
        last_save: UTC::now().timestamp(),
    };



    /*
        Boostrapping process
        During bootstrapping one should query random bootstrap nodes from a
        supplied list; in case where there is no list, rely back on hardcoded
        bootstrap nodes.
        // TODO: actually make it possible to use supplied list; location of a
        //       list should be determined by value supplied in config file;
        //       in case of absence of config file, working dir should be
        //       tried for presence of file named `bootstrap.txt`, only if it
        //       is missing fall back on hardcoded nodes
    */
    bootstrap::bootstrap_hardcoded(&mut tox);

    println!("\nMy ID: {}", tox.get_address());
    println!("My name: {:?}", tox.get_name());

    loop {
        for ev in tox.iter() {
            match ev {
                FriendRequest(fpk, msg) => {
                    on_friend_request(&mut tox, fpk, msg);
                },

                FriendMessage(fnum, _msgkind, msg) => {
                    on_friend_message(&mut tox, fnum, msg, &mut bot);
                },

                _ => println!("Event: {:?}", ev),
            }
        }

        /*
            Mass-spam functionality.
        */
        for fnum in &bot.spam {
            let msg: String = unsafe {
                rand_string((1372.0 * bot.random.gen::<f64>()) as usize,
                            &mut bot.random)
            };

            drop(tox.send_friend_message(*fnum, MessageType::Normal, &msg));
            // ↓ be ready to send message to the next friend on the list
            tox.tick();
        }


        /*
            Write save data every 64s.

            After a write, be it successful or not, set clock again to tick,
            for the next time when it'll need to be saved.
            TODO: save data every $relevant_event, rather than on timer.
        */
        let cur_time = UTC::now().timestamp();
        if bot.last_save + 64 < cur_time {
            match for_files::write_save("bot.tox", tox.save()) {
                Ok(_) => println!("File saved."),
                Err(e) => println!("\nFailed to save file: {}", e),
            }
            bot.last_save = cur_time;
        }

    // needed to run
    tox.wait();
    }
}

