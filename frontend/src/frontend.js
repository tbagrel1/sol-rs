import Vue from 'vue'
import axios from 'axios'
import BootstrapVue from 'bootstrap-vue'
import Vuex from 'vuex'

import 'bootstrap/dist/css/bootstrap.css'
import 'bootstrap-vue/dist/bootstrap-vue.css'

import { status, alerts } from '@/constants'
import App from '@/components/App'

import './globalStyle.styl'

const REFRESH_DELAY = 500

const lexicographicOn = (field) => (obj1, obj2) => {
  return obj1[field].localeCompare(obj2[field])
}
const makeApiUrl = (target) => {
  return `${window.API_ROOT_URL}/${target}`
}
const refreshStateBuilder = (commit, state) => () => {
  if (state.state === null && state.supervisionAlert === null) {
    commit('setSupervisionAlert', alerts.STATE_LOADING)
  }
  return axios.get(makeApiUrl('status'), {
    auth: {
      username: state.username,
      password: state.password
    }
  })
    .then((response) => {
      commit('setState', response.data)
      commit('setSupervisionAlert', alerts.NOTHING)
    })
    .catch((error) => {
      commit('setState', null)
      if (error.response) {
        commit('setSupervisionAlert', alerts.INTERNAL_ERROR)
      } else {
        commit('setSupervisionAlert', alerts.NETWORK_ERROR)
      }
    })
}

Vue.use(Vuex)
Vue.use(BootstrapVue)

// eslint-disable-next-line no-unused-vars
const store = new Vuex.Store({
  state: {
    token: null,
    username: null,
    password: null,
    authenticationAlert: alerts.NOTHING,
    supervisionAlert: alerts.NOTHING,
    state: null
  },
  actions: {
    tryAuthentication: ({ commit, state, dispatch }) => {
      commit('setAuthenticationAlert', alerts.AUTHENTICATION_LOADING)
      return axios.get(makeApiUrl('status'), {
        auth: {
          username: state.username,
          password: state.password
        }
      })
        .then((response) => {
          commit('setAuthenticationAlert', alerts.NOTHING)
          commit('setToken', true)
          dispatch('startRefreshingState')
        })
        .catch((error) => {
          commit('setToken', null)
          if (error.response) {
            if (error.response.status === 401) {
              commit('setAuthenticationAlert', alerts.CREDENTIALS_ERROR)
            } else {
              commit('setAuthenticationAlert', alerts.INTERNAL_ERROR)
            }
          } else {
            commit('setAuthenticationAlert', alerts.NETWORK_ERROR)
          }
        })
    },
    refreshState: ({ commit, state }) => {
      return refreshStateBuilder(commit, state)()
    },
    startRefreshingState: ({ commit, state }) => {
      const refreshState = refreshStateBuilder(commit, state)
      refreshState()
      setInterval(refreshState, REFRESH_DELAY)
    },
    shutdownSingle: ({ commit, state, dispatch }, { group, name }) => {
      return axios.post(makeApiUrl('shutdown'), {
        group_name: group,
        computer_name: name
      }, {
        auth: {
          username: state.username,
          password: state.password
        }
      })
        .then((response) => {
          dispatch('refreshState')
        })
        .catch((error) => {
          if (error.response) {
            commit('setSupervisionAlert', alerts.INTERNAL_ERROR)
          } else {
            commit('setSupervisionAlert', alerts.NETWORK_ERROR)
          }
        })
    },
    shutdownGroup: ({ commit, state, dispatch }, { group }) => {
      return axios.post(makeApiUrl('shutdown'), {
        group_name: group
      }, {
        auth: {
          username: state.username,
          password: state.password
        }
      })
        .then((response) => {
          dispatch('refreshState')
        })
        .catch((error) => {
          if (error.response) {
            commit('setSupervisionAlert', alerts.INTERNAL_ERROR)
          } else {
            commit('setSupervisionAlert', alerts.NETWORK_ERROR)
          }
        })
    }
  },
  mutations: {
    setToken: (state, token) => {
      state.token = token
    },
    setUsername: (state, username) => {
      state.username = username
    },
    setPassword: (state, password) => {
      state.password = password
    },
    setAuthenticationAlert: (state, newAlert) => {
      state.authenticationAlert = newAlert
    },
    setSupervisionAlert: (state, newAlert) => {
      state.supervisionAlert = newAlert
    },
    setState: (state, theState) => {
      state.state = theState
    }
  },
  getters: {
    isAuthenticated: (state, getters) => {
      return state.token !== null
    },
    username: (state, getters) => {
      return state.username
    },
    password: (state, getters) => {
      return state.password
    },
    authenticationAlert: (state, getters) => {
      return state.authenticationAlert
    },
    supervisionAlert: (state, getters) => {
      return state.supervisionAlert
    },
    computers: (state, getters) => (group) => {
      if (state.state === null) {
        return []
      }
      const result = Object.entries(state.state[group]).map(([name, value]) => ({ name: name, status: value.state, fancyStatus: getters.fancyStatus(value.state), group }))
      result.sort(lexicographicOn('name'))
      return result
    },
    groups: (state, getters) => {
      if (state.state === null) {
        return []
      }
      const result = Object.keys(state.state).map(name => ({ name: name, computers: getters.computers(name) }))
      result.sort(lexicographicOn('name'))
      return result
    },
    fancyStatus: (state, getters) => (rawStatus) => {
      switch (rawStatus) {
        case status.ONLINE:
          return 'Allumée'
        case status.SHUTDOWN_REQUESTED:
          return 'Arrêt demandé'
        case status.SHUTDOWN_ACCEPTED:
          return 'Arrêt en cours'
      }
    }
  }
})

new Vue({
  render: createElt => createElt(App),
  store
}).$mount('#app-container')
