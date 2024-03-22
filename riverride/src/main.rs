use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, ScrollUp, SetSize},
    ExecutableCommand, QueueableCommand,
};
use std::{io, time::Duration};

use crossterm::event::{poll, read, Event};

struct World {
    player_c: u16,
    player_l: u16,
}

fn draw(mut sc: &Stdout, world: &World) {
    sc.queue(MoveTo(world.player_c, world.player_l));
    sc.queue(Print("P"));

    sc.flush();
}

fn main() -> std::io::Result<()> {
    //

    // init screen
    let mut sc: Stdout = stdout();
    let (cols, rows) = size()?;
    enable_raw_mode();

    // init player
    let mut world: World = World {
        player_c: (cols / 2),
        player_l: (rows - 1),
    };

    // init the game

    loop {
        // read and apply keyboard

        // pysics

        // draw

        draw(&sc, &world);

        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(10))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            let key = read().unwrap();

            match key {
                Event::Key(event) => match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('l') => {
                        // right
                        if world.player_c < cols - 1 {
                            world.player_c += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        // top
                        if world.player_l > rows / 2 {
                            world.player_l -= 1;
                        }
                    }
                    KeyCode::Char('j') => {
                        // down
                        if world.player_l < rows - 1 {
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
    }

    disable_raw_mode();
    Ok(())
}
