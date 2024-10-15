extern crate find_folder;
extern crate piston_window;
extern crate rand;

use piston_window::*;
use rand::Rng;
use std::collections::LinkedList;

const WIDTH: u32 = 20;
const HEIGHT: u32 = 20;
const SQUARE_SIZE: f64 = 20.0;
const INITIAL_SPEED: f64 = 0.2;

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Snake {
    body: LinkedList<[u32; 2]>,
    dir: Direction,
}

impl Snake {
    fn new(x: u32, y: u32) -> Self {
        let mut body = LinkedList::new();
        body.push_back([x, y]); // Start with a single segment at position (x, y)

        Snake {
            body,
            dir: Direction::Right, // Initial direction
        }
    }

    fn move_forward(&mut self, ate_food: bool) {
        let head = self.body.front().unwrap();
        let mut new_head = *head;

        match self.dir {
            Direction::Up => {
                if head[1] > 0 {
                    new_head[1] -= 1;
                }
            }
            Direction::Down => {
                if head[1] < HEIGHT - 1 {
                    new_head[1] += 1;
                }
            }
            Direction::Left => {
                if head[0] > 0 {
                    new_head[0] -= 1;
                }
            }
            Direction::Right => {
                if head[0] < WIDTH - 1 {
                    new_head[0] += 1;
                }
            }
        }

        self.body.push_front(new_head);

        if !ate_food {
            self.body.pop_back();
        }
    }

    fn grow(&mut self) {
        // We don't pop the tail, effectively growing the snake
    }

    fn check_collision(&self) -> bool {
        let head = self.body.front().unwrap();

        // Check for collisions with the snake's body (ignoring the head)
        for segment in self.body.iter().skip(1) {
            if head == segment {
                return true;
            }
        }

        false
    }
}

struct Food {
    x: u32,
    y: u32,
}

impl Food {
    fn new() -> Food {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..WIDTH);
        let y = rng.gen_range(0..HEIGHT);
        Food { x, y }
    }

    fn respawn(&mut self) {
        let mut rng = rand::thread_rng();
        self.x = rng.gen_range(0..WIDTH);
        self.y = rng.gen_range(0..HEIGHT);
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new(
        "Snake Game",
        [WIDTH * SQUARE_SIZE as u32, HEIGHT * SQUARE_SIZE as u32],
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let ref font = assets.join("FiraSansCondensed-Italic.ttf");
    println!("Loading font from: {:?}", font.display());

    // Create the texture context from the window
    let texture_context = window.create_texture_context();

    // Initialize Glyphs with the texture context
    let mut glyphs = Glyphs::new(font, texture_context, TextureSettings::new()).unwrap();

    let mut snake = Snake::new(WIDTH / 2, HEIGHT / 2);
    let mut food = Food::new();
    let mut game_over = false;
    let mut game_started = false;
    let mut last_update = 0.0;
    let mut speed = INITIAL_SPEED;
    let mut food_eaten = 0;

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            if !game_started {
                if key == Key::Return {
                    game_started = true;
                }
            } else {
                // Change direction based on key press
                snake.dir = match key {
                    Key::Up if snake.dir != Direction::Down => Direction::Up,
                    Key::Down if snake.dir != Direction::Up => Direction::Down,
                    Key::Left if snake.dir != Direction::Right => Direction::Left,
                    Key::Right if snake.dir != Direction::Left => Direction::Right,
                    Key::Return if game_over => {
                        // Reset the game when "Enter" is pressed after game over
                        snake = Snake::new(WIDTH / 2, HEIGHT / 2);
                        food = Food::new();
                        game_over = false;
                        food_eaten = 0;
                        speed = INITIAL_SPEED;
                        game_started = true;
                        snake.dir // Return the current snake direction
                    }
                    _ => snake.dir,
                };
            }
        }

        window.draw_2d(&event, |c, g, _device| {
            clear([0.0, 0.0, 0.0, 1.0], g);

            if !game_started {
                // Draw "Start" button or message
                let transform = c.transform.trans(100.0, 150.0);
                let start_message = "Press Enter to Start";
                let start_color = [1.0, 1.0, 1.0, 1.0]; // white color
                text::Text::new_color(start_color, 32)
                    .draw(start_message, &mut glyphs, &c.draw_state, transform, g)
                    .unwrap();
            } else if !game_over {
                // Draw snake
                for part in &snake.body {
                    rectangle(
                        [0.0, 1.0, 0.0, 1.0],
                        [
                            part[0] as f64 * SQUARE_SIZE,
                            part[1] as f64 * SQUARE_SIZE,
                            SQUARE_SIZE,
                            SQUARE_SIZE,
                        ],
                        c.transform,
                        g,
                    );
                }

                // Draw food
                rectangle(
                    [1.0, 0.0, 0.0, 1.0],
                    [
                        food.x as f64 * SQUARE_SIZE,
                        food.y as f64 * SQUARE_SIZE,
                        SQUARE_SIZE,
                        SQUARE_SIZE,
                    ],
                    c.transform,
                    g,
                );
            } else {
                // Draw "Game Over" message
                let transform = c.transform.trans(100.0, 150.0);
                let game_over_message = "Game Over! Press Enter to Restart";
                let game_over_color = [1.0, 0.0, 0.0, 1.0];
                text::Text::new_color(game_over_color, 32)
                    .draw(game_over_message, &mut glyphs, &c.draw_state, transform, g)
                    .unwrap();
            }
        });

        event.update(|args| {
            if game_started && !game_over {
                if last_update >= speed {
                    let head = snake.body.front().unwrap().clone();
                    let head_x = head[0];
                    let head_y = head[1];

                    let ate_food = head_x == food.x && head_y == food.y;

                    // Move the snake forward and check if it ate food
                    snake.move_forward(ate_food);

                    // If food is eaten, respawn it and grow the snake
                    if ate_food {
                        food.respawn();
                        food_eaten += 1;
                        snake.grow();

                        // Increase speed every 4 foods eaten
                        if food_eaten % 4 == 0 {
                            speed = (speed - 0.02).max(0.05);
                        }
                    }

                    // Check for collisions with the snake or the walls
                    if snake.check_collision() || head_x >= WIDTH || head_y >= HEIGHT {
                        game_over = true;
                    }

                    last_update = 0.0;
                }
                last_update += args.dt;
            }
        });
    }
}
