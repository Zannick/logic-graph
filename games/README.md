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

## YAML file structure

### `Game.yaml`

The file is considered a dictionary, where these are the allowed keys:

* **name**: The name of the game. **Required**.
* **context**: A dictionary of type definitions for context variables. You only need to define something here if you want to use a smaller integer type than 32-bit (smaller is recommended whenever possible), or you want to explicitly define all the options of an enum; the script will make its best guess otherwise based on usage. Each key is the name of the context variable (which must not start with `_`), and its value is another dictionary with these keys:
    * **type**: The name of the type. Valid values are "str", "int", "float", "bool", or any native Rust type. Can be omitted if **max** or **opts** is used. Bools can be defined instead by setting **default**.
    * **max**: The maximum value of the variable. Helpful for using a smaller datatype. Usually means the type is `int`.
    * **opts**: A list of enum names. The variable will always be one of these.
    * **default**: An alternate form of putting the context variable in **start**. For enums this also helps keeps the initial value together with the definition. Can be omitted if your default is 0 or false.
* **start**: A dictionary of initial values of context variables where the type is inferrable from the value (or the variable is defined in **context**). The key **position** is required, and must be set to a **Spot**.
* **load**: A dictionary of values for context variables that will be set whenever the game is *loaded* (by using a warp that loads the game).
* **data**: A dictionary of defaults for Place-based data. Entries are just `key: value`, and the type of the data is inferred from the value, e.g. `0` or `false`. **Spots** can have a value of None, but you have to write `SpotId::None`.
* **objectives**: A dictionary of rules defining what constitutes a "win". **Required**. Each value is a logic rule of type `itemList`. (Due to YAML rules about `[]`s in strings, these rules must be wrapped in `""`.) See the [Logic grammar reference](#logic-grammar-reference) below.
* **movements**: A dictionary of [Movements](#movements). **Required** to have a **default** entry if you want to use movement calculations.
* **time**: A dictionary of tags with default time measurements (as a float in seconds). These tags can be attached to most anything that would have a time value (**Locations**, **Exits**, **Actions**, and **Hybrids**) and if it has no time value, the value defined here is used. The tag **default** represents the fallback if there is no tag and no time.
* **warps**: A dictionary of [Warps](#warps).
* **actions**: A list of the global [Actions](#actions).
* **helpers**: A dictionary of logic helpers. The names of keys must start with `$`. If the helper is not meant to evaluate to a boolean, its type must be specified by adding a `:` followed by the Logic rule name. Helpers can accept arguments, which must be defined in parentheses after the type (if mentioned), with their own types included after a `:`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **collect**: A dictionary of effects that trigger when collecting a specific item. The key is the name of the item, the value is a logic rule of type `action`. Useful when items are permanently collectible but provide currency that can be spent. See the [Logic grammar reference](#logic-grammar-reference) below.
* **settings**: A dictionary of settings that can be changed per-run without having to regenerate the code or recompile the generated code. Keys are the names of the settings, and the values are the same as the **context** fields.
* **special**: A dictionary of special per-game behavior overrides. You can think of these as settings that tweak behavior of the graph analyzer for the game type as a whole, similar to how you provide a settings file when you run the program to tweak behavior of your own graph. There will be a fixed list of these; right now there aren't any.

### Regions

Each other yaml file in the top-level game directory is considered a dictionary that represents a **Region**. Regions may have the following fields:

* **name**: The name of the region.
* **short**: Optionally, a short version of the name. If present, this is the version that will show in most places.
* **data**: A dictionary of values for Place-based data. Format is the same as in `Game.yaml` but the values here override those values, and in turn can be overridden by **data** definitions in more specific places.
* **here**: A dictionary of context variable overrides. The actual values of those variables are ignored and overridden with the given values when the player is in this **Region**.
* **enter**, **load**, **reset**: A dictionary of context variable values to be set on a certain trigger: respectively, whenever the player *enters* the **Region** (i.e. the previous position was not in the **Region** and the new position is), whenever the game is *loaded* (by using a warp that loads the game), and whenever the area is *reset* (via a call to the [built-in function](#built-in-functions) `$reset`). Context variables may be defined here and omitted from `Game.yaml` fields as long as the type is inferrable from the value, and the name does not collide with any other context variable. You may prefix context variables used only in this Region (i.e. *local context variables*) with `_`; other Regions or Places may have a similarly named local context variable without collision.
* **on_entry**: An effect that triggers whenever the player *enters* the Region, similar to **enter** except this field supports deeper customization. The effect is written as a logic rule of type `action`; during this effect's execution, `^position` is the player's previous position, and `^newpos` is the **Spot** the player is moving into. See the [Logic grammar reference](#logic-grammar-reference) below.
* **areas**: A list of [Areas](#areas).

### Areas

**Areas** are defined only within **Regions**. They may have the following fields:

* **name**: The name of the area. Area names must be unique within a Region.
* **data**: A dictionary of values for Place-based data. Format is the same everywhere. These values override the data at higher levels: the containing Region and the defaults defined in `Game.yaml`, and can in turn be overridden by the **data** fields in **Spots**.
* **here**, **enter**, **load**, **reset**, **on_entry**: Same as in **Region** but applying to this **Area** instead.
* **spots**: A list of [Spots](#spots).

### Spots

Spots are only defined within **Areas**. They may have the following fields:

* **name**: The name of the spot. Spot names must be unique within an Area.
* **data**, **here**: Same as in **Areas** but applying/overriding at this **Spot** instead.
* **coord**: A list of coordinates, relative to other Spots in the same **Area**. Only two dimensions are presently supported. Floats are allowed.
* **local**: A list of [Local connections](#local-connections) from this Spot.
* **locations**: A list of [Locations](#locations) accessible from this Spot.
* **exits**: A list of [Exits](#exits) from this Spot.
* **hybrid**: A list of [Hybrid exit-locations](#hybrids) from this Spot.
* **actions**: A list of [Actions](#actions) available at this Spot.

### Movements

Currently, the compiler only understands up to 2-dimensions (and 1-dimension can be represented as 2 trivially). There are three main types of movements: **free**, where the player has a full circle of motion on a plane and can move  (e.g. Ocarina of Time); **xy**, where the player can only move orthogonally in a top-down environment and moving in any dimension is effectively the same speed; and **x** / **y**, where the player has independent speeds for each dimension, e.g. a walking speed and a fall speed.

Movements must define exactly one of those types as a key, and the value is the coordinate-system distance the player can traverse in 1 second. Movements other than **default** must also define **req**: the **requirements** to be able to use the movement, as a logic rule of type `boolExpr`. See the [Logic grammar reference](#logic-grammar-reference) below.

If **x** or **y** is the type of a movement, any of the following fields may also be included:
* **y** or **x**: the other dimension. Keep in mind, though, that this would mean the player can fly indefinitely where the movement is available.
* **fall**: the distance the player can fall in the **y** dimension in 1 second. Note that this field should be negative if `(0, 0)` in your coordinate system is at the bottom rather than the top (i.e. if falling "decreases" the value of **y**).
* **jump**: the time it takes the player to jump and land before jumping again. Local connections that go "up" in **y** value will only be usable with a movement if **y** is defined for the movement, or if **jump** is defined for the movement, and **jumps** is defined for the local connection.
* **jump_down**: the time it takes the player to "jump down". This is just added to the time of any local connections per its total **jumps_down**.

Any field defined in **default** is implicitly available for other movements. Note that you can override the values of any field, but as the **default** movement is considered always available, overriding with a smaller value will not result in limiting movement speed; instead you should make the slower movement the default and invert your rule to have faster movement elsewhere.

Movements that only depend on **data** fields are considered *base movements* because timings based on them can be pre-calculated. The availability of non-base movements must be evaluated during the search, and thus having a large number of them can negatively affect analyzer performance.

### Warps

Warps are always defined globally in `Game.yaml` and are available from any **Spot** (though they can be restricted using **requirements**). They are defined as a dictionary keyed on their **name** and may have the following fields:

* **time**: The time it takes to execute the Warp, in seconds. **Required**; Warps don't presently support tags.
* **req**: The **requirements** to execute the Warp, as a logic rule of type `boolExpr`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **to**: The Warp's destination, either as a specific fixed **Spot** or as a context variable of type **SpotId**. In the former case, the full name of the Spot is required (the region's short name must be used if applicable), with `>` separating the **Region**, **Area**, and **Spot** names: `to: Amagi > Main Area > Save Point`. In the latter case, the context variable is named with a `^` preceding it: `to: ^save`. **Required**.
* **before**: An optional effect that occurs before the player's position is changed (and can thus reference the old spot), as a logic rule of type `action`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **after**: An optional effect that occurs after the player's position is changed, as a logic rule of type `action`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **loads**: If true, after executing this Warp, all context **load** rules will be executed.
* **base_movement**: If true, this Warp is treated as though it is always available for the purposes of time remaining estimation. Only recommended for Menu Warps that are the only method of accessing some locations.

### Local connections

Local connections are always defined in a **Spot**. They may have the following fields:

* **to**: The destination, which must be a **Spot** in the same area. The region name and area name are not required here.
* **thru**: A list of coordinates through which this connection passes. (Note that a single coord is `[x, y]` while a list of a single coord is `[[x, y]]`; the outer `[]` are required for proper YAML parsing.) Effectively this makes this connection a compound connection of multiple lines; the player must be capable of moving through each connection (even if using different movements) in order for the full connection to be usable. (This is handy to avoid creating extra Spots just for moving around an obstacle.)
* **jumps**: A list of the numbers of jumps needed for each individual connection. Must either be unspecified or must have 1 number more than the length of **thru** (note that a single jump may be `1` but multiple jumps must be `[1, 1]`; YAML parses `1, 1` as a string rather than a list). If a connection has **jumps** greater than 0, then the connection's **y** distance is considered feasible with jump movement even if **y** is not defined in an available movement, and regardless of the actual distance.
* **jumps_down**: A list of the numbers of jumps down needed for each individual connection. Must be either unspecified or must have 1 number more than the length of **thru** (similar to **jumps**). This is only used as a delay factor, multiplying by the relevant movement's **jump_down** time.
* **jump_movement**: The name of the movement type required for the **jump** connections. This may eventually be replaced by a list of allowed movement types.

### Locations

### Exits

### Hybrids

### Actions


## Logic grammar reference

### Built-in functions
