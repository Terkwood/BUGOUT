const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

class InvalidLinkModal extends Component {
  constructor() {
    super();
    this.state = { showDialog: false };

    // From GTP.js
    sabaki.events.on("private-game-rejected", () => {
      this.setState({ showDialog: true });
    });
  }

  render({ id = "invalid-link-modal" }) {
    let { showDialog } = this.state;

    let empty = h("div", { id });

    return showDialog
      ? h(
          Dialog,
          {
            id,
            isOpen: true,
          },
          h(Dialog.Header, null, "Invalid Link"),
          h(Dialog.Body, null, "This game is no longer available."),
          h(
            Dialog.Footer,
            null,
            h(
              Dialog.FooterButton,
              {
                accept: true,
                onClick: () => {
                  location.reload();
                },
              },
              "START OVER"
            )
          )
        )
      : empty;
  }
}

export default InvalidLinkModal;
