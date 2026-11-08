package recovery_plan_mesh.recovery_plan
default allow = false
allow if { input.sensitivity != "restricted"; input.separation_of_duty; input.risk <= 0.25 }
