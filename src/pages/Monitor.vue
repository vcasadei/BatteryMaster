<template>
  <q-page class="q-pa-none">
    <q-tabs v-model="tab" dense class="text-grey" active-color="white">
      <q-tab name="full" label="Full" />
      <q-tab name="discharging" label="Discharging" />
      <q-tab name="charging" label="Charging" />
      <q-tab name="empty" label="Critical" />
    </q-tabs>
    <div class="row">
      <div class="col">
        <PercentageGauge
          name="CPU Usage"
          :value="Number((system_store.cpuload * 100).toFixed(1))"
        />
      </div>
      <div class="col">
        <PercentageGauge
          name="Memory Usage"
          :value="
            Number(
              (
                100 -
                (system_store.memfree / system_store.identifier.mem_total) * 100
              ).toFixed(1)
            )
          "
          unit="%"
        />
      </div>
      <div class="col">
        <PercentageGauge
          name="Charge"
          :value="Number((battery_store.percentage! * 100).toFixed(1))"
          unit="%"
        />
      </div>
    </div>
    <q-tab-panels v-model="tab" animated class="bg-dark">
      <q-tab-panel name="full">
        <div class="row">
          <div class="col">
            <PercentageGauge
              name="Battery Health"
              :value="Number((battery_store.state_of_health! * 100).toFixed(1))"
            />
          </div>

          <div class="col">
            <PercentageGauge
              name="Full Capacity"
              :value="Number(battery_store.full_capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="Design Capacity"
              :value="Number(battery_store.design_capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
        </div>
      </q-tab-panel>
      <q-tab-panel name="discharging">
        <div class="row">
          <div class="col">
            <PercentageGauge
              name="Charge/Discharge Power"
              :value="Number(battery_store.energy_rate.toFixed(1))"
              unit="w"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="Battery Voltage"
              :value="Number(battery_store.voltage.toFixed(1))"
              unit="v"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="Current Capacity"
              :value="Number(battery_store.capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
        </div>
      </q-tab-panel>
      <q-tab-panel name="charging">
        <div class="row">
          <div class="col">
            <PercentageGauge
              name="Charge/Discharge Power"
              :value="Number(battery_store.energy_rate.toFixed(1))"
              unit="w"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="Battery Voltage"
              :value="Number(battery_store.voltage.toFixed(1))"
              unit="v"
            />
          </div>
          <div class="col">
            <PercentageGauge
              name="Current Capacity"
              :value="Number(battery_store.capacity!.toFixed(1))"
              unit="wh"
            />
          </div>
        </div>
      </q-tab-panel>
      <q-tab-panel name="empty"> </q-tab-panel>
    </q-tab-panels>
  </q-page>
</template>
<script setup lang="ts">
import { useQuasar } from "quasar";
import { computed, onMounted, ref, watch } from "vue";
import { useStore as useBatteryInfoStore } from "../stores/BatteryInfo";
import { useStore as useSystemInfo } from "../stores/SystemInfo";
import PercentageGauge from "../components/PercentageGauge.vue";
const battery_store = useBatteryInfoStore();
const system_store = useSystemInfo();
const tab = ref(battery_store.state.toLowerCase());
watch(
  () => battery_store.state,
  async (nVal) => {
    if (import.meta.env.MODE != "development") {
      tab.value = nVal.toLowerCase();
    }
  }
);
</script>
