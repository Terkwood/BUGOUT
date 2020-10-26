const { h, Component } = require("preact");
const gametree = require("../modules/gametree");

const ScoreDrawer = require("./drawers/ScoreDrawer");

class DrawerManager extends Component {
  constructor() {
    super();

    this.handleScoreSubmit = ({ resultString }) => {
      this.props.rootTree.nodes[0].RE = [resultString];
      sabaki.closeDrawer();
      setTimeout(() => sabaki.setMode("play"), 500);
    };

    this.handleGameSelect = ({ selectedTree }) => {
      sabaki.closeDrawer();
      sabaki.setMode("play");
      sabaki.setCurrentTreePosition(selectedTree, selectedTree.root.id);
    };

    this.handleGameTreesChange = (evt) => {
      let newGameTrees = evt.gameTrees;
      let { gameTrees, gameCurrents, gameIndex } = this.props;
      let tree = gameTrees[gameIndex];
      let newIndex = newGameTrees.findIndex((t) => t.root.id === tree.root.id);

      if (newIndex < 0) {
        if (newGameTrees.length === 0)
          newGameTrees = [sabaki.getEmptyGameTree()];

        newIndex = Math.min(
          Math.max(gameIndex - 1, 0),
          newGameTrees.length - 1
        );
        tree = newGameTrees[newIndex];
      }

      sabaki.setState({
        gameTrees: newGameTrees,
        gameCurrents: newGameTrees.map((tree, i) => {
          let oldIndex = gameTrees.findIndex((t) => t.root.id === tree.root.id);
          if (oldIndex < 0) return {};

          return gameCurrents[oldIndex];
        }),
      });

      sabaki.setCurrentTreePosition(tree, tree.root.id);
    };
  }

  render({
    mode,
    openDrawer,
    gameTree,

    scoringMethod,
    scoreBoard,
    areaMap,
  }) {
    return h(
      "section",
      {},
      h(ScoreDrawer, {
        show: openDrawer === "score",
        estimating: mode === "estimator",
        areaMap,
        board: scoreBoard,
        method: scoringMethod,
        komi: +gametree.getRootProperty(gameTree, "KM", 0),
        handicap: +gametree.getRootProperty(gameTree, "HA", 0),

        onSubmitButtonClick: this.handleScoreSubmit,
      })
    );
  }
}

module.exports = DrawerManager;
