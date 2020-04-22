const {app, ipcMain, BrowserWindow, Menu} = require('electron')
const {join} = require('path')
const i18n = require('./i18n')
const setting = require('./setting')

let windows = []
let openfile = null
let isReady = false

if (!setting.get('app.enable_hardware_acceleration')) {
    app.disableHardwareAcceleration()
}

function newWindow(path) {
    let window = new BrowserWindow({
        icon: process.platform === 'linux' ? join(__dirname, '..', 'logo.png') : null,
        title: app.getName(),
        useContentSize: true,
        width: setting.get('window.width'),
        height: setting.get('window.height'),
        minWidth: setting.get('window.minwidth'),
        minHeight: setting.get('window.minheight'),
        autoHideMenuBar: !setting.get('view.show_menubar'),
        backgroundColor: '#111111',
        show: false,
        webPreferences: {
            nodeIntegration: true,
            zoomFactor: setting.get('app.zoom_factor')
        }
    })

    windows.push(window)
    buildMenu()

    window.once('ready-to-show', () => {
        window.show()
    })

    window.on('closed', () => {
        window = null
    })

    window.webContents.setAudioMuted(!setting.get('sound.enable'))

    window.webContents.on('new-window', evt => {
        evt.preventDefault()
    })

    window.loadURL(`file://${join(__dirname, '..', 'index.html')}`)

    return window
}

function buildMenu(props = {}) {
    let template = require('./menu').get(props)

    // Process menu items

    let processMenu = items => {
        return items.map(item => {
            if ('click' in item) {
                item.click = () => {
                    let window = BrowserWindow.getFocusedWindow()
                    if (!window) return

                    window.webContents.send(`menu-click-${item.id}`)
                }
            }

            if ('clickMain' in item) {
                let key = item.clickMain

                item.click = () => ({
                    newWindow,
                    checkForUpdates: () => ()
                })[key]()

                delete item.clickMain
            }

            if ('submenu' in item) {
                processMenu(item.submenu)
            }

            return item
        })
    }

    Menu.setApplicationMenu(Menu.buildFromTemplate(processMenu(template)))
}

ipcMain.on('new-window', (evt, ...args) => newWindow(...args))
ipcMain.on('build-menu', (evt, ...args) => buildMenu(...args))

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit()
    } else {
        buildMenu({disableAll: true})
    }
})

app.on('ready', () => {
    isReady = true

    newWindow(openfile)
})

app.on('activate', (evt, hasVisibleWindows) => {
    if (!hasVisibleWindows) newWindow()
})

app.on('open-file', (evt, path) => {
    evt.preventDefault()

    if (!isReady) {
        openfile = path
    } else {
        newWindow(path)
    }
})
