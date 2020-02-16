const status = {
  ONLINE: 'online',
  SHUTDOWN_REQUESTED: 'shutdown_requested',
  SHUTDOWN_ACCEPTED: 'shutdown_accepted'
}

const alerts = {
  NETWORK_ERROR: {
    variant: 'danger',
    message: `Impossible de joindre le serveur pour le moment.`
  },
  INTERNAL_ERROR: {
    variant: 'danger',
    message: 'Erreur interne entre le client web et le serveur de Shutdown On Lan. Veuillez contacter votre administrateur.'
  },
  CREDENTIALS_ERROR: {
    variant: 'danger',
    message: `Nom d'utilisateur ou mot de passe invalide. Réessayez.`
  },
  AUTHENTICATION_LOADING: {
    variant: 'warning',
    message: 'Authentification en cours ...'
  },
  STATE_LOADING: {
    variant: 'warning',
    message: 'Chargement en cours ...'
  },
  NOTHING: null
}

export {
  status,
  alerts
}
