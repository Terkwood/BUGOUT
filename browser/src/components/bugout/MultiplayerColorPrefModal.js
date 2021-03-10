const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

const {
  ColorPref,
  EntryMethod,
  IdleStatus,
} = require("../../modules/multiplayer/bugout");

class MultiplayerColorPrefModal extends Component {
  constructor() {
    super();
    this.state = { showDialog: false, turnedOnOnce: false };

    // From GTP.js
    sabaki.events.on("private-game-rejected", () => {
      this.setState({ showDialog: false, turnedOnOnce: true });
    });
  }

  render({ id = "human-color-pref-modal", data, idleStatus }) {
    if (data == undefined) {
      return h("div", { id });
    }

    let { entryMethod, boardSize } = data;

    let isPrivateEntryWithBoardSizeDefined =
      entryMethod && entryMethod == EntryMethod.CREATE_PRIVATE
        ? boardSize !== undefined
        : true;

    let isBotEntry = entryMethod && entryMethod == EntryMethod.PLAY_BOT;

    let { showDialog, turnedOnOnce } = this.state;

    let turnOn =
      !isBotEntry &&
      ((isPrivateEntryWithBoardSizeDefined && !turnedOnOnce) || showDialog) &&
      idleStatus;

    if (!turnOn) {
      return h("div", { id });
    }

    return h(
      Dialog,
      {
        id,
        isOpen: true,
      },
      h(Dialog.Header, null, "Turn Order"),
      h(
        Dialog.Body,
        null,
        "Choose your color preference. We may assign them at random."
      ),
      h(
        Dialog.Footer,
        null,
        h(
          Dialog.FooterButton,
          {
            accept: true,
            onClick: () => {
              this.setState({ showDialog: false, turnedOnOnce: true });
              sabaki.events.emit("choose-color-pref", {
                colorPref: ColorPref.BLACK,
              });
            },
          },
          "Black"
        )
      ),
      h(
        Dialog.Footer,
        null,
        h(
          Dialog.FooterButton,
          {
            cancel: true,
            onClick: () => {
              this.setState({ showDialog: false, turnedOnOnce: true });
              sabaki.events.emit("choose-color-pref", {
                colorPref: ColorPref.WHITE,
              });
            },
          },
          "White"
        )
      ),
      h(
        Dialog.Footer,
        null,
        h(
          Dialog.FooterButton,
          {
            cancel: true,
            onClick: () => {
              this.setState({ showDialog: false, turnedOnOnce: true });
              sabaki.events.emit("choose-color-pref", {
                colorPref: ColorPref.ANY,
              });
            },
          },
          "Either"
        )
      )
    );
  }
}

export default MultiplayerColorPrefModal;
