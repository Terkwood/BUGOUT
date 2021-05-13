const { shell, clipboard } = require("electron");
const { h, Component } = require("preact");
const classNames = require("classnames");
const { remote } = require("electron");

const TextSpinner = require("../TextSpinner");

const t = require("../../i18n").context("PlayBar");
const helper = require("../../modules/helper");
const setting = remote.require("./setting");

const { Player } = require("../../modules/multiplayer/bugout");

let toggleSetting = (key) => setting.set(key, !setting.get(key));

const isPlayerTurn = (colorNum) => {
  let engineOffset = colorNum === -1 ? 1 : 0;
  return sabaki.state.attachedEngines[engineOffset] == undefined;
};

class PlayBar extends Component {
  constructor() {
    super();
    this.state = { showUndo: false, undoDisabled: true };

    sabaki.events.on("bugout-player-wants-bot", () => {
      this.setState({ showUndo: true });
    });

    // listen for moves, so that we know when to dim the undo button
    sabaki.events.on("they-moved", () => {
      let { showUndo } = this.state;
      if (showUndo) {
        this.setState({ undoDisabled: false });
      }
    });

    // human player moved
    sabaki.events.on("bugout-make-move", () => {
      let { showUndo } = this.state;
      if (showUndo) {
        this.setState({ undoDisabled: true });
      }
    });

    this.handlePassClick = () => {
      if (isPlayerTurn(sabaki.state.currentPlayer)) {
        let autoGenmove = setting.get("gtp.auto_genmove");
        sabaki.makeMove([-1, -1], { sendToEngine: autoGenmove });
      }
    };

    this.handleQuitClick = () => {
      sabaki.makeResign();
      sabaki.setMode("scoring");
    };

    this.handleUndoClick = () => {
      sabaki.undoMove();
    };

    this.handleMenuClick = () => {
      let { left, top } = this.menuButtonElement.getBoundingClientRect();
      helper.popupMenu(
        [
          {
            label: t((p) => `About ${p.appName}…`, { appName: sabaki.appName }),
            click: () =>
              shell.openExternal("https://github.com/Terkwood/BUGOUT"),
          },
          { type: "separator" },
          {
            label: t("Download SGF"),
            click: () => sabaki.saveFile(sabaki.state.representedFilename),
          },
          { type: "separator" },
          {
            label: t("Show &Coordinates"),
            checked: setting.get("view.show_coordinates"),
            click: () => toggleSetting("view.show_coordinates"),
          },
          {
            label: t("Show Move Numbers"),
            checked: setting.get("view.show_move_numbers"),
            click: () => toggleSetting("view.show_move_numbers"),
          },
          {
            label: t("Show Move Colori&zation"),
            checked: setting.get("view.show_move_colorization"),
            click: () => toggleSetting("view.show_move_colorization"),
          },
          { type: "separator" },
          {
            label: t("Es&timate"),
            click: () => sabaki.setMode("estimator"),
          },
          {
            label: t("&Score"),
            click: () => sabaki.setMode("scoring"),
          },
        ],
        left,
        top
      );
    };
  }

  shouldComponentUpdate(nextProps) {
    return nextProps.mode !== this.props.mode || nextProps.mode === "play";
  }

  // balances the undo button
  renderSpacer() {
    return h(
      "a",
      {
        class: "spacer-fake-button",
      },
      h("button", {}, t("AAAA"))
    );
  }

  renderUndo() {
    let { showUndo, undoDisabled } = this.state;
    return showUndo
      ? h(
          "a",
          {
            class: "undo-button",
            onClick: this.handleUndoClick,
          },
          h("button", { disabled: undoDisabled }, t("UNDO"))
        )
      : this.renderSpacer();
  }

  render({
    mode,
    attachedEngines,
    playerBusy,
    playerNames,
    playerRanks,
    playerCaptures,
    currentPlayer,
    showHotspot,
  }) {
    let captureStyle = (index) => ({
      opacity: playerCaptures[index] === 0 ? 0 : 0.7,
    });
    let isEngine = Array(attachedEngines.length).fill(false);

    attachedEngines.forEach((engine, i) => {
      if (engine == null) return;

      playerNames[i] = engine.name;
      playerRanks[i] = null;
      isEngine[i] = true;
    });

    return h(
      "header",
      {
        class: classNames({
          hotspot: showHotspot,
          current: mode === "play",
        }),
      },

      h(
        "a",
        {
          class: "pass-button",
          onClick: this.handlePassClick,
        },
        h("button", {}, t("PASS"))
      ),

      this.renderUndo(),

      h(
        "span",
        { class: "playercontent player_1" },
        h(
          "span",
          { class: "captures", style: captureStyle(0) },
          playerCaptures[0]
        ),
        " ",

        playerRanks[0] &&
          h(
            "span",
            { class: "rank" },
            t((p) => p.playerRank, {
              playerRank: playerRanks[0],
            })
          ),
        " ",

        h(
          "span",
          {
            class: classNames("name", { engine: isEngine[0] }),
            title: isEngine[0] && t("Engine"),
          },
          isEngine[0] && playerBusy[0] && h(TextSpinner),
          " ",
          playerNames[0] || t("Black")
        )
      ),

      h(
        "a",
        {
          class: "current-player",
          title: t("Current Player"),
          ref: (el) => (this.menuButtonElement = el),
          onClick: this.handleMenuClick,
        },
        h("img", {
          src: `./img/ui/player_${currentPlayer}.svg`,
          height: 21,
          alt: t(
            (p) =>
              `${
                p.player < 0 ? "White" : p.player > 0 ? "Black" : p.player
              } to play`,
            { player: currentPlayer }
          ),
        })
      ),

      h(
        "span",
        { class: "playercontent player_-1" },
        h(
          "span",
          {
            class: classNames("name", { engine: isEngine[1] }),
            title: isEngine[1] && t("Engine"),
          },
          playerNames[1] || t("White"),
          " ",
          isEngine[1] && playerBusy[1] && h(TextSpinner)
        ),
        " ",

        playerRanks[1] &&
          h(
            "span",
            { class: "rank" },
            t((p) => p.playerRank, {
              playerRank: playerRanks[1],
            })
          ),
        " ",

        h(
          "span",
          { class: "captures", style: captureStyle(1) },
          playerCaptures[1]
        )
      ),

      this.renderSpacer(),

      h(
        "a",
        {
          class: "quit-button",
          onClick: this.handleQuitClick,
        },
        h("button", {}, t("QUIT"))
      )
    );
  }
}

module.exports = PlayBar;
