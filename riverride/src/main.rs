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

// low number = more speed
const GAME_SPEED: u64 = 200;
// 1~10 (low number = more chance)
const MAP_SHIFT_CHANCE: u16 = 4;
const MAP_GAP: u16 = 10;
const MAP_GENERATE_RATE: u16 = 20;
// 1~10 (low number = more chance)
const ENEMY_RATE: u16 = 8;
const ENEMY_MAX: u16 = 8;
const BULLET_MAX: u16 = 8;
const GOLD_RATE: u16 = 8;
const GOLD_MAX: u16 = 8;
// 1=10% screen .. 100=100% screen
const BULLET_ENERGY: u16 = 80;

struct Enemy {
    l: u16,
    c: u16,
    // life: u16, // if hit bullet-> life -= bullet(power), if life==0-> pop
}
struct Bullet {
    l: u16,
    c: u16,
    // power: u16,
    energy: u16,
}
struct Gold {
    l: u16,
    c: u16,
}
struct World {
    maxC: u16,
    maxL: u16,
    player_c: u16,
    player_l: u16,
    map: Vec<(u16, u16)>,
    died: bool,
    nextStart: u16,
    nextEnd: u16,
    enemy: Vec<Enemy>,
    bullet: Vec<Bullet>,
    gold: Vec<Gold>,
    score: u16,
}

fn draw(mut sc: &mut Stdout, world: &mut World) -> std::io::Result<()> {
    sc.queue(Clear(crossterm::terminal::ClearType::All))?;

    // draw the map
    for l in 0..world.map.len() {
        sc.queue(MoveTo(0, l as u16))?
            .queue(Print("+".repeat(world.map[l].0 as usize)))?
            .queue(MoveTo(world.map[l].1, l as u16))?
            .queue(Print("+".repeat((world.maxC - world.map[l].1) as usize)))?;
    }

    // draw the enemies
    for e in &world.enemy {
        sc.queue(MoveTo(e.c, e.l))?.queue(Print("E"))?;
    }

    // draw the Golds
    for e in &world.gold {
        sc.queue(MoveTo(e.c, e.l))?.queue(Print("G"))?;
    }

    // draw the bullets
    for e in &world.bullet {
        sc.queue(MoveTo(e.c, e.l))?.queue(Print("."))?;
    }

    // draw the player
    sc.queue(MoveTo(world.player_c, world.player_l))?;
    sc.queue(Print("P"))?;

    sc.flush()?;

    Ok(())
}

fn pysics(world: &mut World) {
    // check if player died
    if (world.player_c < world.map[world.player_l as usize].0
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

    // move and add enemies
    if rng.gen_range(0..10) >= ENEMY_RATE && world.enemy.len() < ENEMY_MAX as usize {
        let new_c = rng.gen_range(world.map[0].0..world.map[0].1);
        world.enemy.push(Enemy { c: new_c, l: 0 });
    }
    for i in (0..world.enemy.len()).rev() {
        if world.player_c == world.enemy[i].c && world.player_l == world.enemy[i].l {
            world.died = true;
        }
        for b in (0..world.bullet.len()).rev() {
            if world.enemy[i].c == world.bullet[b].c
                && (world.enemy[i].l == world.bullet[b].l
                    || world.enemy[i].l == world.bullet[b].l + 1)
            {
                world.enemy.remove(i);
                world.bullet.remove(b);
            }
        }

        world.enemy[i].l += 1;
        if world.enemy[i].l > world.maxL - 1 {
            world.enemy.remove(i);
        }
    }

    // move and add Golds
    if rng.gen_range(0..10) >= GOLD_RATE && world.gold.len() < GOLD_MAX as usize {
        let new_c = rng.gen_range(world.map[0].0..world.map[0].1);
        world.gold.push(Gold { c: new_c, l: 0 });
    }
    for i in (0..world.gold.len()).rev() {
        if world.player_c == world.gold[i].c && world.player_l == world.gold[i].l {
            world.gold.remove(i);
            world.score += 1;
        } else {
            world.gold[i].l += 1;
            if world.gold[i].l > world.maxL - 1 {
                world.gold.remove(i);
            }
        }
    }

    // move bullets
    for i in (0..world.bullet.len()).rev() {
        if world.bullet[i].energy == 0 {
            world.bullet.remove(i);
        } else {
            world.bullet[i].l -= 1;
            world.bullet[i].energy -= 1;
            if world.bullet[i].l == 0 {
                world.bullet[i].energy = 0;
            }
        }
    }
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
        enemy: vec![],
        bullet: vec![],
        gold: vec![],
        score: 0,
    };

    // init the game

    while !world.died {
        // read and apply keyboard

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
                    KeyCode::Char(' ') => {
                        // left
                        if world.bullet.len() < BULLET_MAX as usize {
                            world.bullet.push(Bullet {
                                c: world.player_c,
                                l: world.player_l - 1,
                                energy: world.maxL * BULLET_ENERGY / 100,
                            });
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            // Timeout expired and no `Event` is available
        }

        // draw
        draw(&mut sc, &mut world);

        // pysics
        pysics(&mut world);

        let ten_millis = time::Duration::from_millis(GAME_SPEED);
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
