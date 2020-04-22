const { h, Component } = require('preact')

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from 'preact-material-components/Dialog'

const { ConnectionState } = require('../../modules/multiplayer/bugout')

const DisconnectedBody = h(Dialog.Body, null, 'Please wait while we reestablish internet communication.')
const InProgressBody = h(Dialog.Body, null, 'Connecting...')
const ConnectedBody = h(Dialog.Body, null, 'Connected!')
const FailedBody = h(Dialog.Body, null, 'FATAL ERROR ⚰️')

const chooseBody = connectionState => {
    if (connectionState == ConnectionState.DISCONNECTED) {
        return DisconnectedBody
    } else if (connectionState == ConnectionState.IN_PROGRESS) {
        return InProgressBody
    } else if (connectionState == ConnectionState.CONNECTED) {
        return ConnectedBody
    } else {
        return FailedBody
    }
}

class ReconnectModal extends Component {
    constructor() {
        super()
    }

    render({ 
        id = 'reconnect-modal', 
        data
    }) {
        let empty = h('div', { id })

        if (undefined == data) {
            return empty
        }

        let { reconnectDialog, reconnectionState } = data

        if (undefined == reconnectDialog || undefined == reconnectionState) {
            return empty
        }

        return reconnectDialog ?
           h(Dialog,
                {
                    id,
                    isOpen: true,
                },
                h(Dialog.Header, null, 'Dead Net' ),
                chooseBody(reconnectionState)
            )
        : empty
    }
}

export default ReconnectModal
