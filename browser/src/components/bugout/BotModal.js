const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

const { Bot, EntryMethod } = require("../../modules/multiplayer/bugout");

class BotModal extends Component {
  constructor() {
    super();
    this.state = { showDialog: false, turnedOnOnce: false };
  }

  render({ id = "select-bot", data, update }) {
    if (data == undefined) {
      return h("div", { id });
    }

    let { entryMethod } = data;

    let turnOn = entryMethod && entryMethod == EntryMethod.PLAY_BOT;

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
      h(Dialog.Header, null, "Choose Bot"),
      h(Dialog.Body, null, "Lower levels are easier and faster to play."),
      h(
        Dialog.Footer,
        null,
        h(
          Dialog.FooterButton,
          {
            accept: true,
            onClick: () => {
              this.setState({ showDialog: false, turnedOnOnce: true });
              update(Bot.KATAGO_ONE_STAR);
            },
          },
          "KataGo 🌟"
        )
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
              update(Bot.KATAGO_TWO_STARS);
            },
          },
          "KataGo 🌟🌟"
        )
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
              update(Bot.KATAGO_THREE_STARS);
            },
          },
          "KataGo 🌟🌟🌟"
        )
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
              update(Bot.KATAGO_FOUR_STARS);
            },
          },
          "KataGo 🌟🌟🌟🌟"
        )
      )
    );
  }
}

export default BotModal;
