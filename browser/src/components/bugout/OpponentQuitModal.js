const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

class OpponentQuitModal extends Component {
  constructor() {
    super();
    this.state = { showDialog: false, scoringMode: false };

    // From GTP.js
    sabaki.events.on("bugout-opponent-quit", () => {
      this.setState({ showDialog: true });
    });
  }

  render({ id = "opponent-quit-modal" }) {
    let { showDialog } = this.state;

    let empty = h("div", { id });

    return showDialog
      ? h(
          Dialog,
          {
            id,
            isOpen: true,
          },
          h(Dialog.Header, null, "Game Over"),
          h(Dialog.Body, null, "The opponent quit."),
          h(
            Dialog.Footer,
            null,
            h(
              Dialog.FooterButton,
              {
                accept: true,
                onClick: () => {
                  this.setState({ showDialog: false });
                },
              },
              "OK"
            )
          )
        )
      : empty;
  }
}

export default OpponentQuitModal;
