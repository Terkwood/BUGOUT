const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

const {
  BotDifficulty,
  EntryMethod,
} = require("../../modules/multiplayer/bugout");

class BotDifficultyModal extends Component {
  constructor() {
    super();
    this.state = { showDialog: false, turnedOnOnce: false };
  }

  render({ id = "select-bot-difficulty", data, update }) {
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
      h(Dialog.Header, null, "Bot Difficulty"),
      h(
        Dialog.Body,
        null,
        "Choose a difficulty level for the AI. Easier bots take less time to move."
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
              update(BotDifficulty.EASY);
            },
          },
          "Easy 🍼"
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
              update(BotDifficulty.MEDIUM);
            },
          },
          "Medium 🤓"
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
              update(BotDifficulty.HARD);
            },
          },
          "Hard 😈"
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
              update(BotDifficulty.MAX);
            },
          },
          "Max 👹"
        )
      )
    );
  }
}

export default BotDifficultyModal;
