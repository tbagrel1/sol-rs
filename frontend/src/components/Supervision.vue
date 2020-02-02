<template lang="pug">
  b-container.h-100(fluid)
    b-row.h-100
      b-col.h-100.p-0(cols="12")
        b-card.m-4
          b-card-title
            Header
          b-card-body.p-1
            b-container.p-0(v-if="groups.length > 0" fluid)
              Group(v-for="group in groups" :key="group.name" :group="group")
            b-container(v-else)
              b-row
                b-col(cols="12")
                  div Aucune machine connectée
</template>

<script>
import Header from '@/components/Header'
import Group from '@/components/Group'
export default {
  name: 'Supervision',
  components: {
    Header,
    Group
  },
  computed: {
    groups () {
      return this.$store.getters.groups
    }
  }
}
</script>

<style scoped lang="stylus">
</style>
