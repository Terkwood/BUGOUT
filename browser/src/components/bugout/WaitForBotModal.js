const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

class WaitForBotModal extends Component {
  constructor() {
    super();
    this.state = {
      isModalRelevant: false, // does this game even need a bot
      isBotAttached: false, // has the backend signaled that bot is ready to play
      isBotPlaying: false, // is the bot playing the first move
    };

    // From GTP.js
    sabaki.events.on(
      "bugout-wait-for-bot",
      ({ isModalRelevant, isBotAttached, isBotPlaying }) => {
        this.setState({ isModalRelevant, isBotAttached, isBotPlaying });
      }
    );
  }

  render({ id = "wait-for-bot-modal" }) {
    let empty = h("div", { id });

    let message = this.state.isBotPlaying
      ? "KataGo is playing."
      : "Connecting to KataGo...";

    let showDialog =
      this.state.isModalRelevant &&
      (!this.state.isBotAttached || this.state.isBotPlaying);

    return showDialog
      ? h(
          Dialog,
          {
            id,
            isOpen: true,
          },
          h(Dialog.Header, null, "Please Wait"),
          h(Dialog.Body, null, message),
          h(Dialog.Footer, null)
        )
      : empty;
  }
}

export default WaitForBotModal;
