# Cabra

![build workflow status](https://github.com/imatpot/cabra/actions/workflows/build.yml/badge.svg)
![nightly workflow status](https://github.com/imatpot/cabra/actions/workflows/nightly.yml/badge.svg)

Cabra (Spanish for goat) is an AI player for the board game [Caminos][caminos]
(sometimes equated with [Bridget][bridget]), a 3D connection game designed by
[Stefan Kögl][koegl]. The name is a reference to the goat anology in the
[copy of the game's instructions I received][bridget-rules], where valid bridges
are described with 'a goat taking a stroll'.

Cabra uses [Monte Carlo Tree Search][mcts] (MCTS, see also the
[beginner's guide][mcts-beginner]) to determine its moves, and is designed to be
a strong opponent for human players. It is still a work in progress.

## Getting Started

Cabra is implemented in Rust and can be built from source using [Cargo][cargo]
or [Nix][nix], or you can make use of the OCI image available on
[GitHub Container Registry][ghcr].

### Building with Cargo

To build Cabra using Cargo, ensure you have Rust and Cargo installed on your
system. You can then build Cabra like any other Rust project by running either

```bash
cargo build
cargo build --release
```

### Building with Nix

If you have Nix installed (and [Flakes][flake] enabled), you can build Cabra
using the provided [`flake.nix`](flake.nix) file. Clone the repository and
either build or run Cabra with

```bash
nix build
nix run
```

You can also enter a Nix shell with Cabra's development toolchain by running

```bash
nix develop
```

All of these command also work without cloning the repository by referncing the
GitHub repository directly, e.g.

```bash
nix run github:imatpot/cabra
```

### Using the OCI image

> [!NOTE]
> There is currently no stable releases of Cabra, and as such, only the
> `nightly` tag is available.

An OCI image of Cabra is available on GitHub Container Registry. You can pull
the image using Docker or any other OCI-compliant container runtime using e.g.

```bash
docker pull ghcr.io/imatpot/cabra:latest
podman pull ghcr.io/imatpot/cabra:latest
```

You can then run the container with e.g.

```bash
docker run --rm ghcr.io/imatpot/cabra:latest --help
podman run --rm ghcr.io/imatpot/cabra:latest --help
```

as you would with any other container.

## Piece Notation

In order to describe the pieces and their orientations, a notation system has
been defined. Each piece is represented as rotation of a canonical orientation,
and the position of the piece on the board.

### Canonical Rotations

For each of the four piece types — L, T, Z, and O — a canonical rotation has
been defined. This servers as the reference point for all other orientations of
that piece. The canonical rotations for each piece are defined as follows:

```
  L        T        Z        O

█ █ █    █ █ █    █ █       █ █
█          █        █ █     █ █
```

These canonical orientations are defined with the pieces "lying flat" on the X-Y
plane, with the origin (0, 0, 0) in the top left corner. As such, X denotes the
horizontal axis, Y the vertical axis, and Z the depth (or in this case, height)
axis.

### Rotation and Position Notation

To describe the orientation of a piece, we use a notation system that combines
the piece type, the rotation, and the position on the board. The notation is
as follows:

```
<Piece Type> <Rotation> <Position>
```

#### Piece Type

The piece type is denoted by a single letter: `L`, `T`, `Z`, or `O`.

#### Rotation

The rotation is described by a direction in 3D space (X, Y, or Z) and a number
indicating the number of clockwise 90-degree rotations in that direction. The
directions correspond to the 6 faces of a cube and are denoted as follow:

- Top (positive Z-axis): `T`
- Bottom (negative Z-axis): `B`
- North (negative Y-axis): `N`
- South (positive Y-axis): `S`
- East (negative X-axis): `E`
- West (positive X-axis): `W`

The canonical orientation of a piece is denoted by `T0`, indicating that it is
facing the top face and has not been rotated. T, Z, and O have varying degrees
of symmetry. For rotations with equivalent views, Top, North, and East are
preferred.

The full list of possible rotations per piece is as follows:

- L (24):
  - `T0`, `T90`, `T180`, `T270`;
  - `B0`, `B90`, `B180`, `B270`;
  - `N0`, `N90`, `N180`, `N270`;
  - `S0`, `S90`, `S180`, `S270`;
  - `E0`, `E90`, `E180`, `E270`;
  - `W0`, `W90`, `W180`, `W270`.
  - No symmetry.

- T (12): <!-- T/B same, N/S same, E/W same -->
  - `T0`, `T90`, `T180`, `T270`;
  - `N0`, `N90`, `N180`, `N270`;
  - `E0`, `E90`, `E180`, `E270`.
  - Top/Bottom, North/South, and East/West are redundant.

- Z (12): <!-- 0/180 same, 90/270 same -->
  - `T0`, `T90`;
  - `B0`, `B90`;
  - `N0`, `N90`;
  - `S0`, `S90`;
  - `E0`, `E90`;
  - `W0`, `W90`.
  - 0/180 and 90/270 degrees are redundant.

- O (3): <!-- T/B same, N/S same, E/W same, no rotations -->
  - `T0`, `N0`, `E0`.
  - Top/Bottom, North/South, and East/West are redundant.
  - 0/90/180/270 degrees are all redundant.

#### Position

The position of a piece is represented as a triple XYZ coordinate, indicating
the position of the piece's closest cell to the origin (0, 0, 0) on the board.
As the board is only 8x8x3 cells, no spaces are needed.

## Data File Format

Placement data can be stored in text files (typically ending in `.caminos`)
in a compact text-based format.
Each line in a file represents a single placement of four connected cells,
in alternating turns starting with player 1.

### Format Specification

Each placement is encoded as exactly **12 consecutive digits**,
representing the coordinates of four cells.
Any non-digit characters are ignored.
So a line can look something like this, if you want to get creative:

```toml
XYZXYZXYZXYZ
XYZ XYZ XYZ XYZ
x:X y:Y z:Z x:X y:Y z:Z x:X y:Y z:Z x:X y:Y z:Z   # i'm dizzy!
```

But:
- Each group of 3 consecutive digits represents one cell's coordinates: `XYZ`
- `X` and `Y` values must be in the range `0-7` (board size)
- `Z` values must be in the range `0-2` (board height)
- All 12 digits must be present on each line

### Comments

Lines may also include trailing comments separated by a `#` character.
Comments could for example contain the human-readable piece notation:

```toml
120 121 122 110   # L E180 110
```

## About Caminos

Caminos is a two-player abstract strategy game played on a 3D board. Each player
has a set of pieces in their colour, and they take turns placing them on the
board. The goal of Caminos is to connect two opposite sides of the board with a
continuous path of your own pieces. The game is won by the first player to
successfully create a path connecting their designated sides of the board. If no
player can achieve this, the player with the least pieces touching the bottommost edge of
the board wins.

The full rules of the game can be found in the
[Caminos game instructions][caminos-rules]. Note that they slightly differ from
the [rules of Bridget][bridget-rules], but the core mechanics are the same.

[caminos]: https://boardgamegeek.com/boardgame/84913/caminos
[caminos-rules]: https://www.braendi-shop.ch/shop/resources/downloads/SB.A07-02/Caminos_-_Spielanleitung_22_(web).pdf
[bridget]: https://boardgamegeek.com/boardgame/286904/bridget
[bridget-rules]: https://jpneto.github.io/world_abstract_games/modern_rules/2013_Bridget.pdf
[koegl]: https://boardgamegeek.com/boardgamedesigner/2286/stefan-kogl
[mcts]: https://ieeexplore.ieee.org/document/6145622
[mcts-beginner]: https://int8.io/monte-carlo-tree-search-beginners-guide/
[cargo]: https://doc.rust-lang.org/cargo/
[nix]: https://nix.dev/manual/nix/stable/introduction.html
[ghcr]: https://github.com/imatpot/cabra/pkgs/container/cabra
[flake]: https://wiki.nixos.org/wiki/Flakes
