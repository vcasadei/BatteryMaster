<template>
  <q-page>
    <q-form @submit="onSubmit" class="text-grey-2" :loading="loading">
      <q-list bordered padding class="text-grey-3">
        <q-item tag="label" v-ripple>
          <q-item-section>
            <q-item-label>Auto start</q-item-label>
            <q-item-label caption class="text-grey-5"
              >Start with the operating system</q-item-label
            >
          </q-item-section>
          <q-item-section side top>
            <q-toggle v-model="form_value.auto_start" />
          </q-item-section>
        </q-item>
        <q-item tag="label" v-ripple>
          <q-item-section>
            <q-item-label>Start minimized</q-item-label>
            <q-item-label caption class="text-grey-5"
              >Minimize to taskbar on startup</q-item-label
            >
          </q-item-section>
          <q-item-section side top>
            <q-toggle v-model="form_value.start_minimize" />
          </q-item-section>
        </q-item>
        <q-separator spaced />

        <q-item tag="label" v-ripple>
          <q-item-section>
            <q-item-label>Record history</q-item-label>
            <q-item-label caption class="text-grey-5"
              >Record battery consumption history</q-item-label
            >
          </q-item-section>
          <q-item-section side top>
            <q-toggle v-model="form_value.record_battery_history" />
          </q-item-section>
        </q-item>
        <q-separator spaced />

        <q-item-label header class="text-grey-3">Background update interval</q-item-label>
        <q-item>
          <q-item-section side>
            <q-icon color="primary" name="schedule" size="md" />
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.service_update"
              :min="1"
              :max="5"
              marker-labels
              label
            />
          </q-item-section>
        </q-item>
        <q-item-label header class="text-grey-3">UI update interval</q-item-label>
        <q-item>
          <q-item-section side>
            <q-icon color="primary" name="schedule" size="md" />
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.ui_update"
              :min="1"
              :inner-min="form_value.service_update"
              :max="5"
              marker-labels
              label
            />
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section>
            <q-btn
              color="grey"
              label="Discard"
              icon="restart_alt"
              :loading="loading"
              @click="reset"
            ></q-btn>
          </q-item-section>
          <q-item-section>
            <q-btn
              color="primary"
              label="Save"
              icon="save"
              :loading="loading"
              @click="onSubmit"
            ></q-btn>
          </q-item-section>
        </q-item>
      </q-list>
    </q-form>
  </q-page>
</template>
<script setup lang="ts">
import { useQuasar } from "quasar";
import { ref } from "vue";
import { useStore as useConfig, Config } from "../stores/Config";
const $q = useQuasar();
const config_store = useConfig();
const form_value = ref(config_store.$state);
const loading = ref(false);
const onSubmit = async () => {
  loading.value = true;
    try {
    await config_store.update(form_value.value);
    $q.dialog({ message: `Save completed` });
  } catch (err) {
    $q.dialog({ message: `Save failed, error: ${err}` });
  }
  loading.value = false;
};
const reset = async () => {
  loading.value = true;
  const nVal = await config_store.load();
  form_value.value = nVal;
  loading.value = false;
};
</script>
