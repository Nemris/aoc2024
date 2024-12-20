#![warn(clippy::pedantic)]

use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::result;

type Result<T> = result::Result<T, Error>;

/// Possible errors for this program.
#[derive(Debug, PartialEq)]
enum Error {
    InvalidTile,
    NoGuard,
    InfiniteLoop,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidTile => write!(f, "invalid tile"),
            Self::NoGuard => write!(f, "no guard in tiles"),
            Self::InfiniteLoop => write!(f, "infinite loop detected"),
        }
    }
}

impl error::Error for Error {}

/// A single tile in a map.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Ignored,
    Occupied,
    Guard(Direction),
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self> {
        match c {
            '.' => Ok(Tile::Ignored),
            '#' => Ok(Tile::Occupied),
            '^' | 'v' | '<' | '>' => Ok(Tile::Guard(c.try_into().unwrap())),
            _ => Err(Error::InvalidTile),
        }
    }
}

/// A guard patrolling a map.
#[derive(Clone, Debug, Default, PartialEq)]
struct Guard {
    // Direction the guard is walking in.
    direction: Direction,
    // Current position.
    position: usize,
    /// Visited tiles. The first one is the starting position.
    visited: Vec<usize>,
    // Obstacles encountered and the direction they were approached in.
    obstacles: HashMap<usize, HashSet<Direction>>,
}

impl Guard {
    /// Attempts to detect a guard in `map`.
    fn find(map: &Map) -> Option<Self> {
        let obstacles = map.count_obstacles();
        for (i, t) in map.tiles.iter().enumerate() {
            if let Tile::Guard(d) = t {
                return Some(Guard {
                    direction: *d,
                    position: i,
                    visited: Vec::with_capacity(map.tiles.len() - obstacles),
                    obstacles: HashMap::with_capacity(obstacles),
                });
            }
        }
        None
    }

    /// Patrols `map` until `self` exits the room from an edge.
    ///
    /// # Errors
    ///
    /// If an infinite loop is detected, an error is returned.
    fn patrol(&mut self, map: &Map) -> Result<()> {
        loop {
            let offset = self.compute_offset(map);
            let Some(next_pos) = self.position.checked_add_signed(offset) else {
                break;
            };

            if self.is_out_of_bounds(next_pos, map) {
                // The guard exits the room.
                self.visited.push(self.position);
                break;
            }
            if map.tiles[next_pos] == Tile::Occupied {
                // The guard bumps on an obstacle.
                // Bail if an infinite loop is detected.
                self.log_obstacle(next_pos, self.direction)?;
                self.turn();
                continue;
            }

            self.visited.push(self.position);
            self.position = next_pos;
        }

        Ok(())
    }

    /// Returns the coordinates of unique tiles visited.
    fn unique_visits(&self) -> HashSet<usize> {
        self.visited.iter().copied().collect::<HashSet<_>>()
    }

    /// Turns `self` clockwise by one step.
    fn turn(&mut self) {
        self.direction = match self.direction {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    /// Checks if `next_pos` is out of `map`'s bounds, either horizontally or vertically.
    fn is_out_of_bounds(&self, next_pos: usize, map: &Map) -> bool {
        match self.direction {
            Direction::Up | Direction::Down => next_pos >= map.tiles.len(),
            Direction::Left | Direction::Right => next_pos / map.width != self.position / map.width,
        }
    }

    /// Logs an obstacle's position and the direction it was approached in.
    ///
    /// # Errors
    ///
    /// If two obstacles have the same position and are approached from the same direction, then an infinite loop is found and an error is returned.
    fn log_obstacle(&mut self, pos: usize, direction: Direction) -> Result<()> {
        let entry = self.obstacles.entry(pos).or_default();
        if entry.insert(direction) {
            Ok(())
        } else {
            Err(Error::InfiniteLoop)
        }
    }

    /// Computes the offset to reach the next `map` tile in `self.direction`.
    fn compute_offset(&self, map: &Map) -> isize {
        #[allow(clippy::cast_possible_wrap)]
        match self.direction {
            Direction::Up => -(map.width as isize),
            Direction::Down => map.width as isize,
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = Error;

    fn try_from(c: char) -> Result<Self> {
        match c {
            '^' => Ok(Direction::Up),
            'v' => Ok(Direction::Down),
            '<' => Ok(Direction::Left),
            '>' => Ok(Direction::Right),
            _ => Err(Error::InvalidTile),
        }
    }
}

/// A map of tiles, with a guard on patrol.
#[derive(Debug)]
struct Map {
    tiles: Vec<Tile>,
    width: usize,
}

impl Map {
    /// Creates a new `Map` from a newline-separated string.
    fn new(s: &str) -> Result<Self> {
        let tiles: Vec<Vec<Tile>> = s
            .split('\n')
            .map(|s| s.chars().map(Tile::try_from).collect())
            .collect::<result::Result<Vec<_>, _>>()?;
        let width = tiles[0].len();

        let tiles: Vec<Tile> = tiles.into_iter().flatten().collect();
        Ok(Self { tiles, width })
    }

    /// Returns the amount of obstacles in `self`.
    fn count_obstacles(&self) -> usize {
        self.tiles.iter().filter(|&t| *t == Tile::Occupied).count()
    }
}

fn count_loops<I>(tiles: I, map: &mut Map) -> Result<usize>
where
    I: IntoIterator<Item = usize>,
{
    let base_guard = Guard::find(map).ok_or(Error::NoGuard)?;
    let mut loops = 0;
    for tile in tiles {
        if tile == base_guard.position {
            continue;
        }

        let mut guard = base_guard.clone();
        map.tiles[tile] = Tile::Occupied;
        if let Err(Error::InfiniteLoop) = guard.patrol(map) {
            loops += 1;
        }
        map.tiles[tile] = Tile::Ignored;
    }

    Ok(loops)
}

fn main() -> result::Result<(), Box<dyn error::Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let data = fs::read_to_string(dataset)?;

    let mut map = Map::new(&data)?;
    let mut guard = Guard::find(&map).ok_or(Error::NoGuard)?;

    guard.patrol(&map)?;
    println!("Visited tiles: {}", guard.unique_visits().len());

    println!(
        "Infinite loops: {}",
        count_loops(guard.unique_visits(), &mut map)?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Map {
        let s: String = vec![
            "....#.....\n",
            ".........#\n",
            "..........\n",
            "..#.......\n",
            ".......#..\n",
            "..........\n",
            ".#..^.....\n",
            "........#.\n",
            "#.........\n",
            "......#...",
        ]
        .into_iter()
        .collect();

        Map::new(&s).unwrap()
    }

    fn get_looping_map() -> Map {
        let s: String = vec![
            "....#.....\n",
            ".........#\n",
            "..........\n",
            "..#.......\n",
            ".......#..\n",
            "..........\n",
            ".#.#^.....\n",
            "........#.\n",
            "#.........\n",
            "......#...",
        ]
        .into_iter()
        .collect();

        Map::new(&s).unwrap()
    }

    #[test]
    fn guard_finds_own_position_in_map() {
        let m = get_test_data();
        let g = Guard::find(&m);
        assert_eq!(
            g,
            Some(Guard {
                direction: Direction::Up,
                position: 64,
                ..Default::default()
            })
        );
    }

    #[test]
    fn guard_traverses_non_looping_map() {
        let m = get_test_data();
        let mut g = Guard::find(&m).unwrap();

        assert!(g.patrol(&m).is_ok());
    }

    #[test]
    fn guard_detects_infinite_loop() {
        let m = get_looping_map();
        let mut g = Guard::find(&m).unwrap();

        assert!(g.patrol(&m).is_err());
    }

    #[test]
    fn map_counts_visited_tiles() {
        let m = get_test_data();
        let mut g = Guard::find(&m).unwrap();
        g.patrol(&m).unwrap();

        assert_eq!(g.unique_visits().len(), 41);
    }

    #[test]
    fn map_returns_correct_obstacle_count() {
        let m = get_test_data();
        assert_eq!(m.count_obstacles(), 8);
    }

    #[test]
    fn possible_infinite_loops_are_found() {
        let mut m = get_test_data();
        let mut g = Guard::find(&m).unwrap();
        g.patrol(&m).unwrap();

        let unique_tiles = g.visited.iter().copied().collect::<HashSet<_>>();
        assert_eq!(count_loops(unique_tiles, &mut m), Ok(6));
    }
}
