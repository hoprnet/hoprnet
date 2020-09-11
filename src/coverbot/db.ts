import admin, { ServiceAccount } from 'firebase-admin'
import serviceAccount = require('./service-account.json')

// Initialize the app with a service account, granting admin privileges
admin.initializeApp({
  credential: admin.credential.cert(serviceAccount as ServiceAccount),
  databaseURL: 'https://hopr-coverbot.firebaseio.com',
})

// As an admin, the app has access to read and write all data, regardless of Security Rules
const db = admin.database()

export default db
