const { h, Component } = require('preact')

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from 'preact-material-components/Dialog'

class YourColorChosenModal extends Component {
    constructor() {
        super()
        this.state = { showDialog: false, turnedOnOnce: false }

        // From GTP.js
        sabaki.events.on('they-moved',
            () => this.setState({
                showDialog: false, 
                turnedOnOnce: true 
            }))
    }

    render({ id = "your-color-modal", yourColor }) {
        let { showDialog, turnedOnOnce } = this.state
      
        let empty = h('div', { id })

        if (undefined == yourColor || yourColor.wait || undefined == yourColor.event.yourColor) {
            return empty
        }

        let isItReallyOn = !yourColor.wait && (!turnedOnOnce || showDialog)

        if (isItReallyOn && yourColor.event.yourColor === "WHITE") {
            // App.js is waiting on this to potentially call this.generateMove for white
            sabaki.events.emit('your-color', yourColor.event)
        }

        return isItReallyOn ? h(Dialog,
            {
                id,
                isOpen: true,
            },
            h(Dialog.Header, null, 'Your Color'),
            h(Dialog.Body, null, `Please enjoy playing ${yourColor.event.yourColor}.`),
            yourColor.event.yourColor === "BLACK" ?
                h(Dialog.Footer, null, 
                    h(Dialog.FooterButton, 
                        { 
                            accept: true, 
                            onClick: () => {
                                this.setState({showDialog: false, turnedOnOnce: true })
                            }
                        }, 
                        "OK")
                    ) :
                empty,
        ) : empty
    }
}


export default YourColorChosenModal
