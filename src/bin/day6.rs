#![warn(clippy::pedantic)]

use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::result;

type Result<T> = result::Result<T, Error>;

/// Possible errors for this program.
#[derive(Debug)]
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
    Visited,
    Occupied,
    Guard(Direction),
}

impl From<Tile> for char {
    fn from(t: Tile) -> Self {
        match t {
            Tile::Ignored => '.',
            Tile::Visited => 'X',
            Tile::Occupied => '#',
            Tile::Guard(d) => d.into(),
        }
    }
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self> {
        match c {
            '.' => Ok(Tile::Ignored),
            'X' => Ok(Tile::Visited),
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
    // Obstacles encountered and the direction they were approached in.
    obstacles: HashMap<usize, HashSet<Direction>>,
}

impl Guard {
    /// Attempts to detect a guard in `map`.
    fn find(map: &Map) -> Option<Self> {
        for (i, t) in map.tiles.iter().enumerate() {
            if let Tile::Guard(d) = t {
                return Some(Guard {
                    direction: *d,
                    position: i,
                    obstacles: HashMap::new(),
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
    fn patrol(&mut self, map: &mut Map) -> Result<()> {
        while self.position < map.tiles.len() {
            self.patrol_line(map)?;
        }
        Ok(())
    }

    /// Performs patrolling of the area between `self` and the next obstacle.
    ///
    /// Once an obstacle is found, `self` rotates clockwise by 90 degrees.
    ///
    /// # Errors
    ///
    /// If an infinite loop is detected, an error is returned.
    fn patrol_line(&mut self, map: &mut Map) -> Result<()> {
        let offset = self.compute_offset(map);

        while let Some(next_pos) = self.position.checked_add_signed(offset) {
            // The guard exits the room.
            if next_pos >= map.tiles.len() {
                map.tiles[self.position] = Tile::Visited;
                self.position = next_pos;
                break;
            }

            // The guard bumps on an obstacle.
            if map.tiles[next_pos] == Tile::Occupied {
                // Bail if an infinite loop is detected.
                self.log_obstacle(next_pos, self.direction)?;

                self.turn();
                map.tiles[self.position] = Tile::Guard(self.direction);
                break;
            }

            // All good, keep going.
            map.tiles[self.position] = Tile::Visited;
            self.position = next_pos;
        }

        Ok(())
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

impl From<Direction> for char {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Up => '^',
            Direction::Down => 'v',
            Direction::Left => '<',
            Direction::Right => '>',
        }
    }
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
    height: usize,
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::with_capacity(self.width * self.height + self.height - 1);
        for r in self.tiles.chunks_exact(self.width) {
            for c in r.iter().map(|&t| t.into()) {
                s.push(c);
            }
            s.push('\n');
        }
        write!(f, "{s}")
    }
}

impl Map {
    /// Creates a new `Map` from a newline-separated string.
    fn new(s: &str) -> Result<Self> {
        let tiles: Vec<Vec<Tile>> = s
            .split('\n')
            .map(|s| s.chars().map(Tile::try_from).collect())
            .collect::<result::Result<Vec<_>, _>>()?;
        let width = tiles[0].len();
        let height = tiles.len();

        let tiles: Vec<Tile> = tiles.into_iter().flatten().collect();
        Ok(Self {
            tiles,
            width,
            height,
        })
    }

    /// Counts the amount of tiles visited by a `Guard`.
    fn visited_tiles(&self) -> usize {
        self.tiles.iter().filter(|&t| *t == Tile::Visited).count()
    }
}

fn main() -> result::Result<(), Box<dyn error::Error>> {
    let dataset = aoc2024::get_dataset(&PathBuf::from(file!()), "input.txt");
    let data = fs::read_to_string(dataset)?;

    let mut map = Map::new(&data)?;
    let mut guard = Guard::find(&map).ok_or(Error::NoGuard)?;

    guard.patrol(&mut map)?;
    println!("Visited tiles: {}", map.visited_tiles());

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
                obstacles: HashMap::new()
            })
        );
    }

    #[test]
    fn guard_traverses_non_looping_map() {
        let mut m = get_test_data();
        let mut g = Guard::find(&m).unwrap();

        assert!(g.patrol(&mut m).is_ok());
    }

    #[test]
    fn guard_detects_infinite_loop() {
        let mut m = get_looping_map();
        let mut g = Guard::find(&m).unwrap();

        assert!(g.patrol(&mut m).is_err());
    }

    #[test]
    fn map_counts_visited_tiles() {
        let mut m = get_test_data();
        let mut g = Guard::find(&m).unwrap();
        g.patrol(&mut m).unwrap();

        assert_eq!(m.visited_tiles(), 41);
    }
}
