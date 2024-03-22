// use core::time;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ScrollUp, SetSize},
    ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Result, Stdout, Write};
use std::{io, thread, time};

use crossterm::event::{poll, read, Event};

struct World {
    maxC: u16,
    maxL: u16,
    player_c: u16,
    player_l: u16,
    map: Vec<(u16, u16)>,
    died: bool,
}

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
    for l in (0..world.map.len() - 1).rev() {
        world.map[l + 1] = world.map[l]
    }
    if world.map[0].0 > 2 {
        world.map[0] = (world.map[0].0 - 1, world.map[0].1 + 1);
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
        map: vec![(maxC / 2 - 5, maxC / 2 + 5); maxL as usize],
        died: false,
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
