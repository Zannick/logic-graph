# Games content

This directory contains the per-game graph definitions. The folder `sample` is contains an example, while `templates` contains the Jinja2 templates used to generate the automatic code and data that will appear once you run the Compiler.py script on the game. Every other folder should be named after the game it contains in some way.

## Main concepts

The world of the game is represented by these main components:
*   **Item**: A permanent upgrade that can be collected or a permanent change to the world (sometimes called an "event"). Some speedrun categories require collecting specific ones before beating the game.
*   **Location**: the Place that contains an **Item**, and a rule restricting when the player is allowed to acquire that item. Can also be given a **canonical** name in case there are multiple ways or places that can access the in-game item. Can only be visited once.
*   **Spot**: the main graph node type, representing a place the player can be. Can contain **Locations**, **Exits** to other Spots, **Local** movement connections to other Spots, and **Actions** that the player can perform.
*   **Area**: a collection of **Spots**, which share a relative coordinate system used for determining movement time (so that it's not required to time every possible pair of Spots).
*   **Region**: a collection of **Areas**. This mostly serves an organizational purpose.
*   **Exit**: a graph edge detailing what a player needs to move from one **Spot** to another and how long it takes to move in that way.
*   **Hybrid**: an **Exit** that contains a **Location**, essentially an edge that can be traversed multiple times where the player collects an **Item** on the first traversal.
*   **Local**: a graph edge between two **Spots** in the same area, detailing some info that can be used to calculate movement times.
*   **Action**: a thing the player can do that makes temporary changes to the player or the world, and can be done multiple times (resources and abilities permitting). Some of these can be done anywhere, but most will be defined inside a **Spot**.
*   **Warp**: a travel option that can be initiated from anywhere under certain conditions to a fixed or changeable **Spot**.

And finally,
*   **context**: the temporary state of the game, such as whether doors are opened or closed, where the last save was, whether the player is young or old, small or big, etc. This gets combined with the permanent state (items collected, locations visited, etc) to form the full point-in-time state of a playthrough (which is called **Context** throughout the Rust code).

## Folder organization

Generally, there are 6 folders inside your game folder to be aware of: the top-level, `tests`, `benches`, `bin`, `data`, and `src`. The first two will contain files that you edit yourself, the latter four contain only generated files.

You may also wish to create a folder to hold your settings files, since these are also yaml files, but the Compiler.py script will interpret all yaml files at the top-level to be part of the graph definition. Commonly the folder name is `settings`.

### Editable files

In the top-level game directory, you will need to create `Game.yaml` and any other `.yaml` files you like. The first will contain the game-wide definitions you need, while the others define Regions for your game. Based on these, the Compiler script will generate Rust code that you can build, run, benchmark, and potentially test.

In the optional `tests` directory, you can create yaml files that describe graph traversal unittests to ensure that the graph data you've provided works as expected. You can also hand-write your own Rust test cases here, just be careful to avoid naming it `foo_tests.rs` when you have a test file using the name `FooTests`.

### Generated files

The script will create a `Cargo.toml` file at the top-level directory for your game. This is required to build and run the Rust program, so it recommended that you do not touch it.

The `src` directory will contain Rust files that implement the graph for your specific game. The `bin` directory contains the main program starting point used with `cargo run`. The `benches` directory contains the benchmark program used with `cargo bench` that will run some generic tests on your graph.

The `data` directory will contain diagram files for your game, currently a graphviz (dot) file and a mermaid file. GitHub can automatically render the mermaid file, but the interface may be a little tough to use with the typical graph size.

Finally, if you created any test yaml files in `tests`, the same directory will contain the Rust files that run the tests you described.

## `Game.yaml` structure

## Region `.yaml` structure

Each yaml file in the top-level directory other than `Game.yaml` is used to define a Region.


## Logic grammar reference
