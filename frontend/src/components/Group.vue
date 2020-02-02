<template lang="pug">
  b-row
    b-col(cols="12")
      b-card
        b-card-title
          b-container
            b-row
              b-col(cols="6")
                h2 {{ group.name }}
              b-col(cols="3")
                div {{ group.computers.length }} machines
              b-col(cols="3")
                b-btn(:disabled="noneOnline" @click="shutdown") Arrêter le groupe
        b-card-body
          b-container
            Computer(v-for="computer in group.computers" :key="`${group.name}/${computer.name}`" :computer="computer")
</template>

<script>
import { status } from '../constants'
import Computer from '@/components/Computer'
export default {
  name: 'Group',
  components: {
    Computer
  },
  props: {
    group: {
      type: Object,
      required: true
    }
  },
  computed: {
    noneOnline () {
      for (const computer of this.group.computers) {
        if (computer.status === status.ONLINE) {
          return false
        }
      }
      return true
    }
  },
  methods: {
    shutdown () {
      return this.$store.dispatch('shutdownGroup', { group: this.group.name })
    }
  }
}
</script>

<style scoped lang="stylus">

</style>
