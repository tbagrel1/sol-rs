<template lang="pug">
  b-container.h-100.p-0(fluid)
    b-row.h-100.m-0
      b-col.h-100.p-0
        b-card.h-100
          b-card-body.h-100
            b-container.p-0(fluid)
              b-row.m-0
                b-col.p-0
                  b-container.p-0(fluid)
                    b-row
                      b-col.center-inside(cols="6")
                        h2 {{ group.name }}
                      b-col.center-inside(cols="3")
                        div {{ group.computers.length }} machines
                      b-col.center-inside(cols="3")
                        b-btn(:disabled="noneOnline" @click="shutdown" block :variant="noneOnline ? 'secondary' : 'primary'") Arrêter le groupe
              b-row.m-0
                b-col.p-0
                  b-card.mt-3
                    b-card-body.p-0
                      b-container.p-0(fluid)
                        b-row(v-for="computer in group.computers" :key="`${group.name}/${computer.name}`")
                          b-col
                            Computer(:computer="computer")
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
