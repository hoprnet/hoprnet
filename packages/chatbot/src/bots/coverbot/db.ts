import admin, { ServiceAccount } from 'firebase-admin'

const firebasePrivateKey = Buffer.from(process.env.FIREBASE_PRIVATE_KEY, 'base64').toString()

// Initialize the app with a service account, granting admin privileges
admin.initializeApp({
  credential: admin.credential.cert({
    projectId: process.env.FIREBASE_PROJECT_ID,
    clientEmail: process.env.FIREBASE_CLIENT_EMAIL,
    // https://stackoverflow.com/a/41044630/1332513
    privateKey: firebasePrivateKey.replace(/\\n/g, '\n'),
    type: process.env.FIREBASE_TYPE,
    privateKeyId: process.env.FIREBASE_KEY_ID,
    clientId: process.env.FIREBASE_CLIENT_ID,
    authUri: process.env.FIREBASE_AUTH_URI,
    tokenUri: process.env.FIREBASE_TOKEN_URI,
    authProviderX509CertUrl: process.env.FIREBASE_AUTH_PROVIDER_X509_CERT_URL,
    clientX509CertUrl: process.env.FIREBASE_CLIENT_X509_CERT_URL,
  } as ServiceAccount),
  databaseURL: process.env.FIREBASE_DATABASE_URL,
})

// As an admin, the app has access to read and write all data, regardless of Security Rules
const db = admin.database()

export default db
