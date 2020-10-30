const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

const { ColorPref, EntryMethod } = require("../../modules/multiplayer/bugout");

class PlayBotColorSelectionModal extends Component {
  constructor() {
    super();
    this.state = { showDialog: false, turnedOnOnce: false };
  }

  render({ id = "play-bot-color-selection", data }) {
    if (data == undefined) {
      return h("div", { id });
    }

    let { entryMethod, bot } = data;

    let turnOn = entryMethod && entryMethod == EntryMethod.PLAY_BOT && bot;

    let { showDialog, turnedOnOnce } = this.state;

    let happyTimes = (turnOn && !turnedOnOnce) || showDialog;

    if (!happyTimes) {
      return h("div", { id });
    }

    return h(
      Dialog,
      {
        id,
        isOpen: true,
      },
      h(Dialog.Header, null, "Turn Order"),
      h(Dialog.Body, null, "Please choose which color you'd like to play."),
      h(
        Dialog.Footer,
        null,
        h(
          Dialog.FooterButton,
          {
            accept: true,
            onClick: () => {
              this.setState({ showDialog: false, turnedOnOnce: true });
              sabaki.events.emit("human-color-selected", {
                humanColor: ColorPref.BLACK,
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
              sabaki.events.emit("human-color-selected", {
                humanColor: ColorPref.WHITE,
              });
            },
          },
          "White"
        )
      )
    );
  }
}

export default PlayBotColorSelectionModal;
