const { h, Component } = require('preact')

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from 'preact-material-components/Dialog'

class OpponentPassedModal extends Component {
    constructor() {
        super()
        this.state = { showDialog: false, scoringMode: false }

        // From GTP.js
        sabaki.events.on('bugout-move-made',
            ({coord}) => {
                if (coord == undefined) {
                    this.setState({ showDialog: true })
                }
            }
        )
    }

    render({ id = 'opponent-passed-modal' }) {
        let { showDialog } = this.state
      
        let empty = h('div', { id })

        return showDialog ? h(Dialog,
            {
                id,
                isOpen: true,
            },
            h(Dialog.Header, null, 'Your Turn'),
            h(Dialog.Body, null, 'The opponent passed.'),
            h(Dialog.Footer, null, 
                h(Dialog.FooterButton, 
                    { 
                        accept: true, 
                        onClick: () => {
                            this.setState({showDialog: false})
                        }
                    }, 
                    'OK')
                ),
        ) : empty
    }
}


export default OpponentPassedModal
