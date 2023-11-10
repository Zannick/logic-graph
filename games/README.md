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
*   **Local connection**: a graph edge between two **Spots** in the same area, detailing some info that can be used to calculate movement times.
*   **Action**: a thing the player can do that makes temporary changes to the player or the world, and can be done multiple times (resources and abilities permitting). Some of these can be done anywhere, but most will be defined inside a **Spot**.
*   **Warp**: a travel option that can be initiated from anywhere under certain conditions to a fixed or changeable **Spot**.

And finally,
*   **context**: the temporary state of the game, such as whether doors are opened or closed, where the last save was, whether the player is young or old, small or big, etc. This gets combined with the permanent state (items collected, locations visited, etc) to form the full point-in-time state of a playthrough (which is called **Context** throughout the Rust code).
*   **data**: miscellaneous information that can be used like **context** but is constant based on the player position (Place).

## Folder organization

Generally, there are 6 folders inside your game folder to be aware of: the top-level, `tests`, `benches`, `bin`, `data`, and `src`. The first two will contain files that you edit yourself, the latter four contain only generated files.

You may also wish to create a folder to hold your settings files, since these are also yaml files, but the Compiler.py script will interpret all yaml files at the top-level to be part of the graph definition. Commonly the folder name is `settings`.

### Editable files

In the top-level game directory, you will need to create `Game.yaml` and any other `.yaml` files you like. The first will contain the game-wide definitions you need, while the others define Regions for your game. Based on these, the Compiler script will generate Rust code that you can build, run, benchmark, and test.

In the `tests` directory, you can create yaml files that describe graph traversal unittests to ensure that the graph data you've provided works as expected, and run with `cargo test`. Modifying these test files will not require rerunning the build script or recompiling with Rust, since they are parsed directly from Rust. You can also hand-write your own Rust test cases here (obviously these will be recompiled when you run `cargo test`).

### Generated files

The script will create a `Cargo.toml` file at the top-level directory for your game. This is required to build and run the Rust program, so it recommended that you do not touch it.

The `src` directory will contain Rust files that implement the graph for your specific game. The `bin` directory contains the main program starting point used with `cargo run`. The `benches` directory contains the benchmark program used with `cargo bench` that will run some generic tests on your graph.

The `data` directory will contain diagram files for your game, currently a graphviz (dot) file and a mermaid file. GitHub can automatically render the mermaid file, but the interface may be a little tough to use with the typical graph size. The graphviz file can be rendered with `neato` to produce a 2D map of your spots based on their coordinates; you may have to adjust the Game's `graph_scale_factor` and some regions' `graph_offset` to make some spots visible. You can then re-scale and overlay the produced image onto a map image with [GraphicsMagick](http://graphicsmagick.org), e.g. `gm composite -geometry 5801x -geometry +241+168 digraph.png map.png digraph-map.png` (you'll have to calculate your sizes and offsets).

Finally, there will be a `tests` directory with a `unittest.rs` file. This test file will run any YAML test cases you put in that directory.

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
* **special**: A dictionary of special per-game behavior overrides. You can think of these as settings that tweak behavior of the graph analyzer or renderer for the game type as a whole, similar to how you provide a settings file when you run the program to tweak behavior of your own graph. There is a fixed list of these.
    * **graph_scale**: A pair of float multipliers to apply to your coordinate system when rendering the graph in graphviz (via neato). Note that graphviz considers the origin (0,0) to be in the lower left. If your coordinate system puts the origin in a different corner, you will want a negative multiplier so that your graph is not mirrored, e.g. `[180, -100]` for an origin in the upper left. 0 is not a valid multiplier. If you're using **map_file**, I suggest using the number of pixels per grid unit.
    * **map_file**: The filename of an image which you'd like your graph overlaid on, which should be placed in the `data/` directory. PNG files recommended.
    * **map_ppi**: The pixels per inch for your map file, used to render a graph at the same size.
    * **map_min_coord**: The min x/y values in your coordinate system, corresponding to the edges of your map image. That is, if you extend your grid to the corners of the image, what are the minimum values? Must be specified alongside **map_max_coord** to be used.
    * **map_max_coord**: The max x/y values in your coordinate system, corresponding to the edges of your map image. That is, if you extend your grid to the corners of the image, what are the maximum values? Must be specified alongside **map_min_coord** to be used.

### Regions

Each other yaml file in the top-level game directory is considered a dictionary that represents a **Region**. Regions may have the following fields:

* **name**: The name of the region.
* **short**: Optionally, a short version of the name. If present, this is the version that will show in most places.
* **graph_offset**: Optionally, a pair of floats used to offset where this region is placed in the graphviz generated graph. No effect on the game graph itself.
* **graph_attrs**: Optionally, a string of graphviz node attributes to apply to spots within this region. No effect on the game graph itself.
* **data**: A dictionary of values for Place-based data. Format is the same as in `Game.yaml` but the values here override those values, and in turn can be overridden by **data** definitions in more specific places.
* **here**: A dictionary of context variable overrides. The actual values of those variables are ignored and overridden with the given values when the player is in this **Region**.
* **enter**, **load**, **reset**: A dictionary of context variable values to be set on a certain trigger: respectively, whenever the player *enters* the **Region** (i.e. the previous position was not in the **Region** and the new position is), whenever the game is *loaded* (by using a warp that loads the game), and whenever the area is *reset* (via a call to the [built-in function](#built-in-functions) `$reset` or whenever the game is *loaded*). Context variables may be defined here and omitted from `Game.yaml` fields as long as the type is inferrable from the value, and the name does not collide with any other context variable. You may prefix context variables used only in this Region (i.e. *local context variables*) with `_`; other Regions or Places may have a similarly named local context variable without collision.
* **on_entry**: An effect that triggers whenever the player *enters* the Region, similar to **enter** except this field supports deeper customization. The effect is written as a logic rule of type `action`; during this effect's execution, `^position` is the player's previous position, and `^newpos` is the **Spot** the player is moving into. See the [Logic grammar reference](#logic-grammar-reference) below.
* **areas**: A list of [Areas](#areas).

### Areas

**Areas** are defined only within **Regions**. They may have the following fields:

* **name**: The name of the area. Area names must be unique within a Region. **Required**.
* **data**: A dictionary of values for Place-based data. Format is the same everywhere. These values override the data at higher levels: the containing Region and the defaults defined in `Game.yaml`, and can in turn be overridden by the **data** fields in **Spots**.
* **here**, **enter**, **load**, **reset**, **on_entry**, **graph_offset**, **graph_attrs**: Same as in **Region** but applying to this **Area** instead.
* **spots**: A list of [Spots](#spots).

### Spots

Spots are only defined within **Areas**. They may have the following fields:

* **name**: The name of the spot. Spot names must be unique within an Area. **Required**.
* **data**, **here**, **graph_offset**, **graph_attrs**: Same as in **Areas** but applying/overriding at this **Spot** instead.
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
* **req**: The **requirements** to execute the Warp, as a logic rule of type `boolExpr`. If omitted, functions the same as `True`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **price**: The numerical value of the **Currency** required to be spent. If unset, executing the Warp is considered free.
* **costs**: The name of the **Currency** required to be spent. Any global context variable with an integer value is considered eligible Currency for this. If omitted and **price** is set, the first one defined in `Game.yaml` **context** is used.
* **to**: The Warp's destination, either as a specific fixed **Spot** or as a context variable of type **SpotId**. In the former case, the full name of the Spot is required (the region's short name must be used if applicable), with `>` separating the **Region**, **Area**, and **Spot** names: `to: Amagi > Main Area > Save Point`. In the latter case, the context variable is named with a `^` preceding it: `to: ^save`. **Required**.
* **before**: An optional effect that occurs before the player's position is changed (and can thus reference the old spot), as a logic rule of type `action`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **after**: An optional effect that occurs after the player's position is changed, as a logic rule of type `action`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **loads**: If true, after executing this Warp, all context **load** rules will be executed.
* **base_movement**: If true, this Warp is treated as though it is always available for the purposes of time remaining estimation. Only recommended for Menu Warps that are the only method of accessing some locations.

### Local connections

Local connections are always defined in a **Spot**. They may have the following fields:

* **to**: The destination, which must be a **Spot** in the same area. The region name and area name are not required here.
* **thru**: A list of coordinates through which this connection passes. (Note that a single coord is `[x, y]` while a list of a single coord is `[[x, y]]`; the outer `[]` are required for proper YAML parsing.) Effectively this makes this connection a compound connection of multiple lines; the player must be capable of moving through each connection (even if using different movements) in order for the full connection to be usable. (This is handy to avoid creating extra Spots just for moving around an obstacle.)
* **jumps**: A list of the numbers of jumps needed for each individual connection. Must either be unspecified or must have 1 number more than the length of **thru** (note that a single jump may be `1` but multiple jumps must be `[1, 1]`; YAML parses `1, 1` as a string rather than a list). If a connection has **jumps** greater than 0, then the connection's **y** distance is considered feasible with jump movement even if **y** is not defined in an available movement, and regardless of the actual distance (unless the distance is considered a fall).
* **jumps_down**: A list of the numbers of jumps down needed for each individual connection. Must be either unspecified or must have 1 number more than the length of **thru** (similar to **jumps**). This is only used as a delay factor, multiplying by the relevant movement's **jump_down** time.
* **jump_movement**: The name of the movement type required for the **jump** connections. This may eventually be replaced by a list of allowed movement types.

### Locations

Locations are always defined in a **Spot**. They may have the following fields:

* **name**: The name of the Location. Location names must be unique within a Spot. **Required**.
* **item**: The id of the Item. This may only use alphanumeric characters and underscores and must start with a capital letter. **Required**.
* **canon**: The canonical name of the Location. All Locations with the same canonical name are considered alternative ways to access the same logical item in the game; after visiting any of these, all the rest with the same canonical name will be skipped. All Locations with the same canonical name must have the same Item.
* **req**: The **requirements** to visit the Location, as a logic rule of type `boolExpr`. If omitted, functions the same as `True`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **price**: The numerical value of the **Currency** required to be spent. If unset, accessing the Location is considered free.
* **costs**: The name of the **Currency** required to be spent. Any global context variable with an integer value is considered eligible Currency for this. If omitted and **price** is set, the first one defined in `Game.yaml` **context** is used.
* **time**: The time it takes to access the Location.
* **tags**: A list of string tags for the Location, which may be used to set a default time, or to mark certain groups of Locations. If multiple tags have times associated with them, the largest is chosen by default.

### Exits

Exits are always defined in a **Spot**. They may have the following fields:

* **to**: The destination **Spot**. If the Spot is in the same Region, the Region may be omitted, e.g. `to: Main Area > Save Point`. If the Spot is in the same Area, both the Region and Area may be omitted, e.g. `to: Ledge`.
* **req**: The **requirements** to take the Exit, as a logic rule of type `boolExpr`. If omitted, functions the same as `True`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **price**: The numerical value of the **Currency** required to be spent. If unset, taking the Exit is considered free.
* **costs**: The name of the **Currency** required to be spent. Any global context variable with an integer value is considered eligible Currency for this. If omitted and **price** is set, the first one defined in `Game.yaml` **context** is used.
* **time**: The time it takes to take the Exit.
* **tags**: A list of string tags for the Exit, which may be used to set a default time, or to mark certain groups of Exits. If multiple tags have times associated with them, the largest is chosen by default.
* **movement**: A single movement type (or `base`), which is used to calculate the time as though the exit is a local movement between the two spots. Does not currently support **thru**. If **time** is set, this has no effect.
* **jumps**: Similar to **jumps** for [local connections](#local-connections), a single number used to calculate as the number of jumps necessary to traverse the **y** distance. Only considered when using **movement** to set time.
* **jumps**: Similar to **jumps_down** for [local connections](#local-connections), a single number used as a delay factor for falling down the **y** distance. Only considered when using **movement** to set time.

### Hybrids

Hybrids are always defined in a **Spot**. They have the same fields as both Locations and Exits with the following caveats:

* **req** is used for both **Location** and **Exit**.
* **price** and **costs** refer to the **Location**, and as such will only be paid once when the Location is accessed. To set a cost of taking the Exit separately, use **exit_price** and **exit_costs**. If those are left unset, the Exit is considered free (other than the Location visit).
* **time** refers to the **Exit**. To set a time for the **Location** access, use **item_time**.
* **tags** are used to set a default time for both **time** and **item_time**. You may override either or both.

### Actions

Actions are always defined in a **Spot**. They may have the following fields:

* **name**: The name of the Action. Action names must be unique within a Spot. **Required**.
* **req**: The **requirements** to perform the Action, as a logic rule of type `boolExpr`. If omitted, functions the same as `True`. See the [Logic grammar reference](#logic-grammar-reference) below.
* **do**: The **effect** of the Action, as a logic rule of type `action`. **Required** (else why have the action?). See the [Logic grammar reference](#logic-grammar-reference) below.
* **price**: The numerical value of the **Currency** required to be spent. If unset, accessing the Location is considered free.
* **costs**: The name of the **Currency** required to be spent. Any global context variable with an integer value is considered eligible Currency for this. If omitted and **price** is set, the first one defined in `Game.yaml` **context** is used.
* **time**: The time it takes to execute the Action.
* **tags**: A list of string tags for the Action, which may be used to set a default time, or to mark certain groups of Actions. If multiple tags have times associated with them, the largest is chosen by default.

## Logic grammar reference

For full and up-to-date details, please consult [the full ANTLRv4 grammar](../grammar/Rules.g4). This is meant as a quick summary.

The logic rules are intended to read like a subset or variant of Python. Each field that expects a logic rule expects a particular **type** of rule, which is the same in general as its **return type**. And logic rules are built out of expressions that expect certain types of expressions and primitives and return their own types. (Note that this is not true typechecking!)

### Main Rules

* **boolExpr**: The type expected in **req** fields. You may wrap a boolExpr in parentheses (`( )`), and you may combine them with boolean logic operators `and` or `or`.
* **actions**: The type expected in **do**, **before**, and **after** fields. These fields expect one or more statements of type **action**, separated by `;` (a semicolon after the last statement is optional).
* **itemList**: The type expected for **objectives**. This is also a boolean expression that can be used in **req** fields. See [Testing Item Possession](#testing-item-possession).

### Primitives

* the values `True` or `False`
* integers
* floats
* an **Item**, written the same as you would find it in [Locations](#locations). Must begin with a capital letter and contains only alphanumerics and underscores.
* a **setting**. Must begin with a lowercase letter and contains only alphanumerics and underscores.
* a **context variable** or helper **argument**. Indicated with a `^`, then must begin with a lowercase letter or underscore, and contains only alphanumerics and underscores.
* a **Place**. Must be enclosed in backticks and fully specified, e.g. `` `Amagi > Main Area > Save Point` ``.

### Expressions

#### Testing Item Possession

Anywhere that expects a **boolExpr** can check whether an item has been collected in any of these manners:

* the Item id, to check for at least one of the item.
* the Item id followed by a number in braces, e.g. `Infect{2}` to check that at least that number copies of that item have been collected. The number must either be a integer literal or a **setting** name.
* 'NOT' followed by the Item id, to check that the item has not ever been collected.
* a **context variable** or helper **argument** of type Item, to check for at least one of that referenced item.
* **itemList**, which checks that all of the possessions in the list are true. Viable options for entries in the list include: any of the above 4 options, or a **helper** function of type **itemList** that takes no arguments.

#### value

A **value** expression is either a **setting**, a **context variable**, or a **helper argument**.

#### num

A **num** primitive is either a integer literal or a **value** expression.

Two expressions of type **num** can be combined with a binary operation of `+`, `-`, `*`, or `/`. Division on integral types is always integer division.

#### str

The grammar does not support building or modifying strings. Instead, to save space, all string values are converted by the Compiler script into enum values for the respective **context variables** that represent some form of mode. Because of this, strings for different variables can't be compared to each other.

The grammar does not presently support a way to reference a specific enum. Instead it accepts string literals, and the enum type is inferred from where the literal is assigned or what it is compared against.

A **str** primitive is either a string literal or a **value** expression.

#### action

(These are usually called *statements* in some programming languages.)

There are only two main **action** primitives:
*  Assignment: You may overwrite the value saved in a **context variable** with `^var = expr`. The expression on the right side is allowed to use `^var` to read the value of the variable before the assignment.
*  Alteration: For numerical types, you can adjust the value in a **context variable** with e.g. `^var += expr`. Allowed operations are `+`, `-`, `*`, and `/`. Division on integral types is always integer division.

The other types of **action** expressions are **function invocations** and **conditionals**.

#### Comparisons

Comparisons are always **boolExpr**. You can compare:

* a **value** expression of numerical type can be compared `==`, `!=`, `>=`, `>`, `<=`, or `<` with a **num** expression.
* a **value** expression of numerical type can be tested as a bitflag containing all the set bits in a **num** expression, by writing `value & num`. This is a comparison rather than a binary operation.
* a **value** expression of string type can be compared `==` or `!=` with a **str** expression.
* a **context variable** or **helper argument** can be compared `==` with an **Item** primitive or a **setting** value (by name). The types should match.
* a **context variable** or **helper argument** of type **Item** can be tested for inclusion in a literal list of **Item** names, by writing `^item_var in [Item1, Item2, ...]`. The list must contain at least two Items, otherwise you should use the `==` comparison.

#### Conditionals

Conditionals are written in the form `IF (boolExpr) { ... } ELSE IF (boolExpr) { ... } ELSE { ... }`. Parentheses and brackets are required. The `...` must all be the same rule (which is the return type of the conditional), and one of:
* **boolExpr**
* **num**
* **str**
* **actions**

You may have as many `ELSE IF` blocks as you like. The `ELSE` block is optional for **boolExpr**, in which case the else case is considered `false`. The `ELSE` block is optional for **actions** as well, which does not need a default.

#### Negation

For these **boolExpr** expressions only can you use `NOT` to negate it:
*  **Item** possessions of one item, e.g. `NOT Item`: tests that the Item has not been collected
*  **value** expressions, e.g. `NOT value`: tests that the expression is false.
*  **function invocations**, e.g. `NOT $func`: tests that the function returns false.
*  **Place containment**, e.g. ``NOT WITHIN `Place` `` or ``^place_var NOT WITHIN `Place` ``: tests that the position or given **Place** variable is not within the other.

#### Switch, Match, and Per

Switch statements (despite the name) all begin with either `MATCH` or `PER`; it does not matter which, though you may prefer them differently for different kinds of statements. They are always written `MATCH thing { case => ..., case2 => ..., _ => ... }`. The final case must always be the catch-all case `_`, even if it's impossible to reach. The return type of the statement is the same type as the `...` expressions which must all be of the same type.

* If `thing` is an **Item** name, the cases must all be integer literals; the case that will be chosen is the number of that Item that have been collected so far. These numbers will influence the upper bound on the number of this Item tracked, even if impossible to collect that many. The return type may be any of **boolExpr**, **num**, or **str**.
* If `thing` is a **setting** name, the cases must all be either integer literals or string literals; the case that will be chosen is the one that matches the value of the setting. The return type may be any of **boolExpr**, **num**, or **str**.
* If `thing` is a **context variable** or **helper argument**, *and* the return type is **boolExpr**, the cases must all be **Item**s, though each case may have multiple Items separated by `|` to indicate that any such Item matches that case.
* If `thing` is a **context variable** or **helper argument**, and the return type is either **num** or **str**, the cases must all be either integer literals or string literals; the case that will be chosen is the one that matches the value of the variable or argument.

These could be changed in the future to make settings and variables work the same way.

#### Function invocations

Function invocations are written `$func(arg1, arg2, ...)`. Function invocations with no arguments provided can be written as just `$func`. Available functions include **helpers** defined in `Game.yaml`, and the following built-in functions:
* **max** and **min**: Type **num**. Returns the **max**imum or **min**imum of the two provided numerical arguments.
* **count**: Type **num**. Accepts one **Item** argument and returns how many of that **Item** have been collected. Note that this may be capped based on the maximum value needed in any rule (if we never check for multiples, this may return 1 even if the item is collected multiple times; if we never check for the Item at all, this always returns 0).
* **default**: Any type that has a Rust default (numbers, Spots, and enums). Returns the default value of that type. Useful mainly for setting a context variable to `SpotId::None` which is not otherwise recognized in this grammar.
<!-- * **all_spot_checks**, **all_area_checks**, **all_region_checks**: Type **boolExpr**. Accepts one **Place** argument that must be a **Spot**, **Area**, or **Region**, respectively, and returns whether all **Locations** in that **Place** have been either visited or skipped. -->
* **get_area**, **get_region**: Type **Place**. Accepts one **Spot** argument and returns the **Area** or **Region**, respectively, that contains the Spot.
* **reset_area**, **reset_region**: Type **action**. Accepts one **Place** argument that must be an **Area** or **Region** respectively, and resets the given **Area** or **Region**. Note that resetting a Region does not reset all the Areas in that Region.
* **skip**: Type **action**. Accepts one **Location** argument and skips it.
* **add_item**: Type **action**. Adds one of the given **Item** to the context without triggering **collect** rules.
* **collect**: Type **action**. Adds one of the given **Item** to the context *and* triggers **collect** rules for that item. *Be careful not to create an infinite loop!*

Functions of type **boolExpr**, **action**, or **Place** may accept any one of these argument sets (no mixing and matching):
* Any number of **Item**s.
* Any number of **Place**s.
* Any number of **value** expressions.
* Any one integer, float, or string literal.
* Any one **context variable** or **helper argument**.
* Nothing.

Function invocations of type **boolExpr** may additionally be negated.

Functions of type **num** may currently accept any one of these argument sets (no mixing and matching):
* A single **Item**.
* Any number of **num** expressions.
* Nothing.

Functions of type **itemList** do not currently allow any arguments.

These restrictions on arguments are possible to change if needed; this is just the current grammar.

#### Place Containment

You can check whether a certain **Place** variable is inside of another **Place** via `p1 WITHIN p2` or `p1 NOT WITHIN p2`:

* `p1` must be a **context variable** or **function argument**, or it may be omitted, in which case the current position is used.
* If `p1` is omitted, `p2` may be either a **Place** literal in backticks, or a tuple (surrounded with `()`) of **Place** literals separated by commas.
* If `p1` is a variable or argument, `p2` may be a **Place** literal, another variable or argument, or a **function invocation** of type **Place**.

These cases could be changed in the future to supply the same behavior if needs arise.
