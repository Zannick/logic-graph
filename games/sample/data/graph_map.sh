# AUTOGENERATED FOR sample - MODIFICATIONS WILL BE LOST
# Requires graphviz (for neato) and GraphicsMagick (for gm)

(
cd "$( dirname -- "${BASH_SOURCE[0]}" )";
echo "Generating graph..." &&
neato -Tpng -o digraph.png digraph.dot &&
echo "Set map_file and map params in Game.yaml under 'special:' to render this graph on the map."
)