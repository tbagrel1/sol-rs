<template lang="pug">
  b-container(v-if="groups.length > 0" fluid)
    b-row.pl-3.pt-3
      b-col.pr-3.pb-3.pl-0.pt-0(v-for="group in groups" :key="group.name" cols="12" lg="6")
        Group(:group="group")
  b-container.mt-3(v-else fluid)
    b-row
      b-col(cols="12")
        b-list-group-item(variant="warning")
          span Aucune machine connectée
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
