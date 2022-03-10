<template>
    <div class="container notification status-box">
        <h2 class="title status-info">
          <b-tooltip v-if="details" :label="details">
            {{ label }}
          </b-tooltip>
          <span v-else>{{ label }}</span>

          <b-loading :active="loading" :is-full-page="false" />

          <b-tooltip :label="info || (working ? 'No issues detected!' : 'Not currently working')">
                <b-icon v-if="loading" icon="help-circle" />
                <span v-else-if="typeof working === 'string'">{{ working }}</span>
                <b-icon v-else-if="working" icon="check-circle" class="has-text-success" />
                <b-icon v-else icon="close-circle" class="has-text-danger" />
          </b-tooltip>
        </h2>
    </div>
</template>

<script>
export default {
  props: {
    working: {
      type: [Boolean, String],
      required: true,
    },
    label: {
      type: String,
      required: true,
    },
    details: {
      type: String,
      default: null,
    },
    info: {
      type: String,
      default: null,
    },
    loading: {
      type: Boolean,
      default: false,
    },
  },
}
</script>

