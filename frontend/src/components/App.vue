<template lang="pug">
  div#app.h-100
    b-navbar(type="dark" variant="primary")
      b-navbar-brand
        h1 Shutdown On Lan
    Authentication(v-if="!isAuthenticated")
    SupervisionAlert(v-else-if="supervisionAlert !== alerts.NOTHING")
    Supervision(v-else)
</template>

<script>
import { alerts } from '../constants'
import SupervisionAlert from './SupervisionAlert'
import Supervision from './Supervision'
import Authentication from './Authentication'
export default {
  name: 'App',
  components: {
    SupervisionAlert,
    Authentication,
    Supervision
  },
  data: () => ({
    alerts
  }),
  computed: {
    isAuthenticated () {
      return this.$store.getters.isAuthenticated
    },
    supervisionAlert () {
      return this.$store.getters.supervisionAlert
    }
  }
}
</script>

<style scoped lang="stylus">
  #app
    display flex
    flex-direction column
</style>
