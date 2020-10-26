const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

const { Visibility } = require("../../modules/multiplayer/bugout");

class WaitForOpponentModal extends Component {
  constructor() {
    super();
    this.state = { copied: false, reconnectedOnce: false };
  }

  updateClipboard(newClip) {
    navigator.clipboard.writeText(newClip).then(
      () => {
        this.setState({ copied: true });
      },
      () => {
        throw Exception("clipboard write failed");
      }
    );
  }

  render({ id = "wait-for-opponent-modal", data, reconnectDialog }) {
    // dfried says a thunk is a thunk is a thunk
    let copyLinkFooter = () =>
      h(
        Dialog.Footer,
        null,
        h(
          Dialog.FooterButton,
          {
            accept: true,
            onClick: () => this.updateClipboard(data.event.link),
          },
          this.state.copied ? "Copied! ⭐︎" : "Copy link 🔗"
        )
      );

    let emptyFooter = () => h(Dialog.Footer, null);

    let isPublic = () =>
      data.hasEvent && data.event.visibility === Visibility.PUBLIC;

    let empty = () => h("div", { id });

    let body = () => {
      if (!this.state.reconnectedOnce && reconnectDialog) {
        this.setState({ reconnectedOnce: true });
      }

      if (reconnectDialog || this.state.reconnectedOnce) {
        // Never show this modal once reconnect procedures have
        // been initiated
        return empty();
      }

      if (data.gap) {
        return h(Dialog.Body, null, "Negotiating game venue...");
      }

      if (isPublic()) {
        return h(
          Dialog.Body,
          null,
          "The game will start once both players are present."
        );
      }

      // private
      return h(
        Dialog.Body,
        null,
        `Click the button below to copy a link to this game onto your clipboard.  You may then paste it to a friend.`
      );
    };

    return undefined != data && (data.gap || data.hasEvent)
      ? h(
          Dialog,
          {
            id,
            isOpen: true,
          },
          h(
            Dialog.Header,
            null,
            isPublic() || data.gap ? "Please Wait" : "Share Private Link"
          ),
          body(),
          isPublic() || data.gap ? emptyFooter() : copyLinkFooter()
        )
      : empty();
  }
}

export default WaitForOpponentModal;
