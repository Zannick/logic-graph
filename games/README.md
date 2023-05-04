# Games content

This directory contains the per-game graph definitions. The folder `sample` is contains an example, while `templates` contains the Jinja2 templates used to generate the automatic code and data that will appear once you run the Compiler.py script on the game. Every other folder should be named after the game it contains in some way.

## Main concepts

The world of the game is represented by these main components:
*   **Item**: A permanent upgrade that can be collected or a permanent change to the world (sometimes called an "event"). Some speedrun categories require collecting specific ones before beating the game.
*   **Places**: representations of pieces of the world
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
*   **data**: miscellaneous information that can be used like **context** but is constant based on the player position (Place).

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

The file is considered a dictionary, where these are the allowed keys:

* **name**: The name of the game. **Required**.
* **context**: A dictionary of type definitions for context variables. You only need to define something here if you want to use a smaller integer type than 32-bit (smaller is recommended whenever possible), or you want to explicitly define all the options of an enum; the script will make its best guess otherwise based on usage. Each key is the name of the context variable, and its value is another dictionary with these keys:
    * **type**: The name of the type. Valid values are "str", "int", "float", "bool", or any native Rust type. Can be omitted if **max** or **opts** is used. Bools can be defined instead by setting **default**.
    * **max**: The maximum value of the variable. Helpful for using a smaller datatype. Usually means the type is `int`.
    * **opts**: A list of enum names. The variable will always be one of these.
    * **default**: An alternate form of putting the context variable in **start**. For enums this also helps keeps the initial value together with the definition. Can be omitted if your default is 0 or false.
* **start**: A dictionary of initial values of context variables where the type is inferrable from the value (or the variable is defined in **context**). The key **position** is required, and must be set to a **Spot**.
* **load**: A dictionary of values of context variables that will be set whenever the game is *loaded* (by using a warp that loads the game).
* **data**: A dictionary of defaults for Place-based data. Entries are just `key: value`, and the type of the data is inferred from the value, e.g. `0` or `false`. **Spots** can have a value of None, but you have to write `SpotId::None`.
* **objectives**: A dictionary of rules defining what constitutes a "win". **Required**. See the [Logic grammar reference](#logic-grammar-reference) below.
* **movements**: A dictionary of movement definitions. **Required** to have a **default** entry if you want to use movements.
    * TODO
* **time**: A dictionary of tags with default time measurements (as a float in seconds). These tags can be attached to anything that would have a time value (**Locations**, **Exits**, **Actions**, **Hybrids**, and **Warps**) and if it has no time value, the value defined here is used. The tag **default** represents the fallback if there is no tag and no time.
* **warps**: A dictionary of the warps.
    * TODO
* **actions**: A list of the actions.
    * TODO
* **helpers**: A dictionary of logic helpers. The names of keys must start with `$`. If the helper is not meant to evaluate to a boolean, its type must be specified by adding a `:` followed by the Logic rule name. Helpers can accept arguments, which must be defined in parentheses after the type (if mentioned), with their own types included after a `:`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **collect**: A dictionary of effects that trigger when collecting a specific item. The key is the name of the item, the value is a logic rule of type `action`. Useful when items are permanently collectible but provide currency that can be spent. See the [Logic grammar reference](#logic-grammar-reference) below.
* **settings**: A dictionary of settings that can be changed per-run without having to regenerate the code or recompile the generated code. Keys are the names of the settings, and the values are the same as the **context** fields.
* **special**: A dictionary of special per-game behavior overrides. You can think of these as settings that tweak behavior of the graph analyzer for the game type as a whole, similar to how you provide a settings file when you run the program to tweak behavior of your own graph. There will be a fixed list of these; right now there aren't any.

## Region `.yaml` structure

Each yaml file in the top-level directory other than `Game.yaml` is used to define a Region.


## Logic grammar reference
