import * as firebase from 'firebase/app'
import 'firebase/database'

const config = {
  apiKey: 'AIzaSyCy8OKn0dcfH9TUFQpULJe2ZvjPRt0Vu08',
  authDomain: 'hoprassociation.firebaseapp.com',
  databaseURL: 'hopr-coverbot.firebaseio.com',
  storageBucket: 'hoprassociation.appspot.com',
}

if (!firebase.apps.length) {
  firebase.initializeApp(config)
}

// Get a reference to the database service
export default firebase.database()
