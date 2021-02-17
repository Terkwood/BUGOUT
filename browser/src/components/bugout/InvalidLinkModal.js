const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

class InvalidLinkModal extends Component {
  constructor() {
    super();
    this.state = { showDialog: false };
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
          h(Dialog.Body, null, "This link is no longer available."),
          h(
            Dialog.Footer,
            null,
            h(
              Dialog.FooterButton,
              {
                accept: true,
                onClick: () => {
                  this.setState({ showDialog: false });
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
