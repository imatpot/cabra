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

## About Caminos

Caminos is a two-player abstract strategy game played on a 3D board. Each player
has a set of pieces in their colour, and they take turns placing them on the
board. The goal of Caminos is to connect two opposite sides of the board with a
continuous path of your own pieces. The game is won by the first player to
successfully create a path connecting their designated sides of the board. If no
player can achieve this, the player with the least pieces touching the edge of
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
