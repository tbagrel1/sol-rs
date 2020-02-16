<template lang="pug">
  b-container.p-0(fluid)
    b-row.mx-0.my-1
      b-col.center-inside.pl-0(cols="6")
        h3.m-0.computer-name {{ computer.name }}
      b-col.center-inside(cols="3")
        div.computer-status {{ computer.fancyStatus }}
      b-col.center-inside.pr-0(cols="3")
        b-btn(:disabled="computer.status !== status.ONLINE" @click="shutdown" block :variant="computer.status !== status.ONLINE ? 'secondary' : 'success'") Arrêter
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
  h3
    font-size large
</style>
