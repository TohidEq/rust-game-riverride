// use core::time;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ScrollUp, SetSize},
    ExecutableCommand, QueueableCommand,
};
use std::{io, thread, time};
use std::{
    io::{stdout, Result, Stdout, Write},
    os::linux::raw::stat,
};

use crossterm::event::{poll, read, Event};

use rand::Rng;

struct World {
    maxC: u16,
    maxL: u16,
    player_c: u16,
    player_l: u16,
    map: Vec<(u16, u16)>,
    died: bool,
    nextStart: u16,
    nextEnd: u16,
}

// 1~10 (low number = more chance)
const MAP_SHIFT_CHANCE: u16 = 4;
const MAP_GAP: u16 = 10;
const MAP_GENERATE_RATE: u16 = 20;

fn draw(mut sc: &Stdout, world: &World) -> std::io::Result<()> {
    sc.queue(Clear(crossterm::terminal::ClearType::All))?;

    // draw the map
    for l in 0..world.map.len() {
        sc.queue(MoveTo(0, l as u16))?;
        sc.queue(Print("+".repeat(world.map[l].0 as usize)))?;

        sc.queue(MoveTo(world.map[l].1, l as u16))?;
        sc.queue(Print("+".repeat((world.maxC - world.map[l].1) as usize)))?;
    }

    // draw the player
    sc.queue(MoveTo(world.player_c, world.player_l))?;
    sc.queue(Print("P"))?;

    sc.flush()?;

    Ok(())
}

fn pysics(mut world: World) -> Result<World> {
    // check if player died
    if (world.player_c <= world.map[world.player_l as usize].0
        || world.player_c >= world.map[world.player_l as usize].1)
    {
        world.died = true;
    }

    // shift the map
    let mut rng = rand::thread_rng();
    // rng.gen_range(0..10)

    for l in (0..world.map.len() - 1).rev() {
        world.map[l + 1] = world.map[l]
    }

    if (world.nextStart == world.map[0].0) {
        //
        let end = world.nextEnd - MAP_GAP;
        let mut start = 0;
        if end > MAP_GENERATE_RATE {
            start = end - MAP_GENERATE_RATE
        }
        world.nextStart = rng.gen_range(start..end);
    }
    if (world.nextEnd == world.map[0].1) {
        //
        let mut end = world.maxC - 1;
        let start = world.nextStart + MAP_GAP;
        if start < ((world.maxC - 1) - MAP_GENERATE_RATE) {
            end = start + MAP_GENERATE_RATE
        }
        world.nextEnd = rng.gen_range((start)..(end));
    }

    if rng.gen_range(0..10) > MAP_SHIFT_CHANCE {
        if (rng.gen_range(0..2) == 1) {
            if world.map[0].0 < world.nextStart {
                world.map[0].0 += 1;
            }
            if world.map[0].0 > world.nextStart {
                world.map[0].0 -= 1;
            }
        } else {
            if world.map[0].1 < world.nextEnd {
                world.map[0].1 += 1;
            }
            if world.map[0].1 > world.nextEnd {
                world.map[0].1 -= 1;
            }
        }
    }

    Ok((world))
}

fn main() -> std::io::Result<()> {
    //

    // init screen
    let mut sc: Stdout = stdout();
    let (maxC, maxL) = size()?;
    enable_raw_mode();
    sc.execute(Hide)?;

    // init player
    let mut world: World = World {
        maxC: maxC,
        maxL: maxL,
        player_c: (maxC / 2),
        player_l: (maxL - 1),
        map: vec![(maxC / 2 - MAP_GAP, maxC / 2 + MAP_GAP); maxL as usize],
        died: false,
        nextStart: maxC / 2 - MAP_GAP - 1,
        nextEnd: maxC / 2 + MAP_GAP + 1,
    };

    // init the game

    while !world.died {
        // read and apply keyboard

        // pysics
        world = pysics(world).unwrap();
        // draw

        draw(&sc, &world);

        // `poll()` waits for an `Event` for a given time period
        if poll(time::Duration::from_millis(10))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            let key = read().unwrap();

            // clear the buffer
            while poll(time::Duration::from_millis(10)).unwrap() {
                let _ = read();
            }

            match key {
                Event::Key(event) => match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('l') => {
                        // right
                        if world.player_c < maxC - 1 {
                            world.player_c += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        // top
                        if world.player_l > maxL / 2 {
                            world.player_l -= 1;
                        }
                    }
                    KeyCode::Char('j') => {
                        // down
                        if world.player_l < maxL - 1 {
                            world.player_l += 1;
                        }
                    }
                    KeyCode::Char('h') => {
                        // left
                        if world.player_c > 1 {
                            world.player_c -= 1;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            // Timeout expired and no `Event` is available
        }

        draw(&sc, &world);
        let ten_millis = time::Duration::from_millis(100);
        let now = time::Instant::now();

        thread::sleep(ten_millis);
    }

    disable_raw_mode();
    sc.execute(Show)?;
    Ok(())
}

// NOTE: Random gen map shift
// NOTE: Enemy
// NOTE: Gas
// NOTE: Gold
// NOTE:
