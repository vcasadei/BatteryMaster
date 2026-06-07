<template>
  <q-page>
    <q-dialog v-model="warn_dialog">
      <q-card style="width: 320px" class="q-pa-xs">
        <q-list separator>
          <q-item dense>
            <q-item-section
              ><q-item-label class="text-h6">Warning</q-item-label>
              <q-item-label caption
                >Some of the power limits you set did not take effect. This may be because your machine does not support certain settings, or we do not support your hardware. Apply power limits with caution; avoid using the lock feature if unsure.</q-item-label
              ></q-item-section
            >
          </q-item>
          <q-item dense
            ><q-item-section
              >Actual queried values after applying the power settings</q-item-section
            ></q-item
          >
          <q-item dense>
            <q-item-section> stapm_limit </q-item-section>
            <q-item-section side> {{ set_result.stapm_limit }}w</q-item-section>
          </q-item>

          <q-item dense>
            <q-item-section> slow_limit </q-item-section>
            <q-item-section side> {{ set_result.slow_limit }}w</q-item-section>
          </q-item>

          <q-item dense>
            <q-item-section> fast_limit </q-item-section>
            <q-item-section side> {{ set_result.fast_limit }}w</q-item-section>
          </q-item>
        </q-list>
          <q-card-actions align="right">
          <q-btn flat @click="warn_dialog = false">Close</q-btn>
        </q-card-actions>
      </q-card>
    </q-dialog>
    <q-banner
      dense
      inline-actions
      class="text-white bg-red"
      v-show="!power_store.isAdmin"
    >
      Requires administrative privileges
      <template v-slot:action>
        <q-btn
          size="sm"
          flat
          color="white"
          label="Run as Admin"
          icon="restart_alt"
        />
      </template>
    </q-banner>
    <q-banner
      dense
      class="text-white bg-warning"
      v-show="sys_store.support_power_set && power_store.isAdmin"
      >This feature is only supported on AMD CPUs (Zen2+). It may damage hardware; use with caution.
    
    }}</q-banner>

    <q-form @submit="onSubmit" :loading="loading">
      <q-list separator>
        <q-item>
          <q-item-section>
            <q-item-label class="text-white">{{
              sys_store.identifier.cpu_vendor
            }}</q-item-label>
            <q-item-label caption class="text-grey"
              >{{ sys_store.identifier.cpu_name }}-{{
                power_store.identifier.cpu_family
              }}</q-item-label
            >
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.stapm_limit"
              show-value
              size="100px"
              color="light-green-6"
              track-color="light-green-2"
              class="text-light-green"
            ></q-knob>
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.stapm_limit"
              :min="5"
              :max="120"
              label
              label-always
              :disable="setting_disabled"
            /><q-item-label class="text-light-green text-h6"
                >Long-term power (W)</q-item-label
            >
              <q-item-label class="text-grey" caption
                >Maximum sustained power limit without hitting thermal walls or other factors;</q-item-label
            >
          </q-item-section>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.stamp_value"
              show-value
              size="100px"
              color="amber-6"
              track-color="amber-2"
              class="text-amber"
            ></q-knob>
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.slow_limit"
              show-value
              size="100px"
              color="light-green-6"
              track-color="light-green-2"
              class="text-light-green"
            ></q-knob>
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.slow_limit"
              :inner-min="form_value.stapm_limit"
              :min="5"
              :max="120"
              label
              label-always
              :disable="setting_disabled"
            /><q-item-label class="text-light-green text-h6"
                >Short-term power (W)</q-item-label
            >
              <q-item-label class="text-grey" caption
                >Maximum power the CPU can sustain for a short period</q-item-label
            >
          </q-item-section>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.slow_value"
              show-value
              size="100px"
              color="amber-6"
              track-color="amber-2"
              class="text-amber"
            ></q-knob>
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.fast_limit"
              show-value
              size="100px"
              color="light-green-6"
              track-color="light-green-2"
              class="text-light-green"
            ></q-knob>
          </q-item-section>
          <q-item-section>
            <q-slider
              v-model="form_value.fast_limit"
              :inner-min="form_value.stapm_limit"
              :min="5"
              :max="120"
              label
              label-always
              :disable="setting_disabled"
            /><q-item-label class="text-light-green text-h6"
                >Instantaneous power (W)</q-item-label
            >
              <q-item-label class="text-grey" caption
                >Peak instantaneous power the CPU can reach</q-item-label
            >
          </q-item-section>
          <q-item-section side>
            <q-knob
              readonly
              v-model="power_store.fast_value"
              show-value
              size="100px"
              color="amber-6"
              track-color="amber-2"
              class="text-amber"
            ></q-knob>
          </q-item-section>
        </q-item>
        <q-item tag="label" v-ripple>
          <q-item-section>
            <q-item-label class="text-white">Auto Lock</q-item-label>
            <q-item-label caption class="text-grey-5"
                >Other processes may reset power limits. When locked, values are restored every 10 seconds.</q-item-label
            >
          </q-item-section>
          <q-item-section side top>
            <q-toggle
              v-model="form_value.auto_lock"
              @update:model-value="auto_lock_change"
              :disable="!power_store.isAdmin || !form_value.modifyed"
            />
          </q-item-section>
        </q-item>
        <q-item>
          <q-item-section>
            <q-btn
              color="grey"
              label="Restore"
              icon="restart_alt"
              :loading="loading"
              @click="onReset"
              :disable="
                btn_disabled || !form_value.modifyed || form_value.auto_lock
              "
            >
              <q-tooltip class="q-pa-none">
                <q-list bordered separator v-if="!!power_store.init_value">
                  <q-item dense>
                    <q-item-section>
                      <q-item-label
                        >Will be reset to the following settings</q-item-label
                      ></q-item-section
                    >
                  </q-item>
                  <q-item dense>
                    <q-item-section avatar>stapm_limit</q-item-section>
                    <q-item-section
                      ><q-item-label>{{
                        power_store.init_value!.stapm_limit
                      }}</q-item-label></q-item-section
                    >
                  </q-item>
                  <q-item dense>
                    <q-item-section avatar>slow_limit</q-item-section>
                    <q-item-section
                      ><q-item-label>{{
                        power_store.init_value!.slow_limit
                      }}</q-item-label></q-item-section
                    >
                  </q-item>
                  <q-item dense>
                    <q-item-section avatar>fast_limit</q-item-section>
                    <q-item-section
                      ><q-item-label>{{
                        power_store.init_value!.fast_limit
                      }}</q-item-label></q-item-section
                    >
                  </q-item>
                </q-list></q-tooltip
              >
            </q-btn>
          </q-item-section>
          <q-item-section>
            <q-btn
              color="primary"
              label="Apply"
              icon="save"
              :loading="loading"
              :disable="btn_disabled || !can_submit || form_value.auto_lock"
              @click="onSubmit"
            ></q-btn>
          </q-item-section>
        </q-item>
      </q-list>
    </q-form>
  </q-page>
</template>
<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useStore as usePower, LimitSet } from "../stores/ApuPower";
import { useStore as userBattery } from "../stores/BatteryInfo.ts";
import { useStore as useSystem } from "../stores/SystemInfo";
import { invoke } from "@tauri-apps/api/core";

import { useQuasar } from "quasar";
const $q = useQuasar();
const power_store = usePower();
power_store.refresh().then();
const sys_store = useSystem();
const loading = ref(false);
const warn_dialog = ref(false);
const enable_autolock = ref(false);

const setting_disabled = computed(() => {
  return (
    form_value.value.auto_lock ||
    !power_store.isAdmin ||
    !sys_store.support_power_set
  );
});
const btn_disabled = computed(() => {
  return !power_store.isAdmin || !sys_store.support_power_set;
});
const can_submit = computed(() => {
  return (
    form_value.value.fast_limit != power_store.fast_limit ||
    form_value.value.slow_limit != power_store.slow_limit ||
    form_value.value.stapm_limit != power_store.stapm_limit
  );
});
const set_result = ref({} as LimitSet);
const set_limit = async (val: LimitSet) => {
  loading.value = true;

  const result = await power_store.set_limit({
    ...val,
  });
  $q.notify(`set apu limit ${result?.[0]}`);
  if (result && !result[0]) {
    set_result.value.fast_limit = result![1].fast_limit;
    set_result.value.slow_limit = result![1].slow_limit;
    set_result.value.stapm_limit = result![1].stapm_limit;
    warn_dialog.value = true;
  } else if (result && result[0]) {
    enable_autolock.value = true;
  }
  loading.value = false;
};
const onSubmit = async () => {
  if (form_value.value.slow_limit < form_value.value.stapm_limit)
    form_value.value.slow_limit = form_value.value.stapm_limit;
  if (form_value.value.fast_limit < form_value.value.stapm_limit)
    form_value.value.fast_limit = form_value.value.stapm_limit;
  await set_limit(form_value.value);
  form_value.value.modifyed = true;
};
const onReset = async () => {
  if (power_store.init_value) {
    await set_limit(power_store.init_value);
    form_value.value.modifyed = false;
  }
};
const form_value = ref(power_store.form_value);
const exec_elevate_self = async () => {
  await invoke("exec_elevate_self");
};
const auto_lock_change = async () => {
  await power_store.set_limit_lock(form_value.value.auto_lock, {
    ...form_value.value,
  } as LimitSet);
};
</script>
