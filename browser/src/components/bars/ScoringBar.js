const { h, Component } = require("preact");
const Bar = require("./Bar");
const t = require("../../i18n").context("ScoringBar");

class ScoringBar extends Component {
  constructor() {
    super();

    this.handleDetailsClick = () => sabaki.openDrawer("score");
    this.handleNewGameClick = () => location.reload();
  }

  render({ type, method, areaMap, scoreBoard, komi, handicap }) {
    let score = scoreBoard && scoreBoard.getScore(areaMap, { komi, handicap });
    let result =
      score && (method === "area" ? score.areaScore : score.territoryScore);

    return h(
      Bar,
      Object.assign({ type }, this.props),
      h(
        "div",
        { class: "result" },
        h("button", { onClick: this.handleDetailsClick }, t("Details")),
        h(
          "strong",
          {},
          !result
            ? ""
            : result > 0
            ? t((p) => `B+${p.result}`, { result })
            : result < 0
            ? t((p) => `W+${p.result}`, { result: -result })
            : t("Draw")
        )
      ),
      " ",
      type === "scoring" ? t("Select dead stones.") : t("Toggle group status."),
      h(
        "button",
        {
          id: "new-game-button",
          onClick: this.handleNewGameClick,
        },
        t("New Game")
      )
    );
  }
}

module.exports = ScoringBar;
