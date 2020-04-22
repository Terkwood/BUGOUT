const uuidv4 = require('uuid/v4')

const storageType = 'localStorage'
const clientIdKey = 'bugoutClientId'

// Provides a randomized, persistent client ID
const fromStorage = () => {
    if (storageAvailable()) {
        var storage;
        try {
            storage = window[storageType]
            var found = storage.getItem(clientIdKey)
            if (found) {
                return found
            } else {
                let newId = uuidv4()
                storage.setItem(clientIdKey, newId)
                return newId
            }
        } catch (_e) {
            return uuidv4()
        }
    }
}

const storageAvailable = () => {
    var storage
    try {
        storage = window[storageType]
        let k = '__storage_test__'
        storage.setItem(k, k)
        storage.removeItem(k)
        return true;
    }
    catch(e) {
        return e instanceof DOMException && (
            // everything except Firefox
            e.code === 22 ||
            // Firefox
            e.code === 1014 ||
            // test name field too, because code might not be present
            // everything except Firefox
            e.name === 'QuotaExceededError' ||
            // Firefox
            e.name === 'NS_ERROR_DOM_QUOTA_REACHED') &&
            // acknowledge QuotaExceededError only if there's something already stored
            (storage && storage.length !== 0)
    }
}

exports.fromStorage = fromStorage 
