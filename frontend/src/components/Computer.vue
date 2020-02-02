<template lang="pug">
  b-row
    b-col
      b-container
        b-row
          b-col(cols="6")
            div.computer-name {{ computer.name }}
          b-col(cols="3")
            div.computer-status {{ computer.fancyStatus }}
          b-col(cols="3")
            b-btn(:disabled="computer.status !== status.ONLINE" @click="shutdown") Arrêter
</template>

<script>
import { status } from '../constants'
export default {
  name: 'Computer',
  props: {
    computer: {
      type: Object,
      required: true
    }
  },
  data: () => ({
    status
  }),
  methods: {
    shutdown () {
      this.$store.dispatch('shutdownSingle', {
        group: this.computer.group,
        name: this.computer.name
      })
    }
  }
}
</script>

<style scoped lang="stylus">

</style>
