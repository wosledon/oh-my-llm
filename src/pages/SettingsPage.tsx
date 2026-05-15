import { useEffect } from 'react';
import {
  Title1,
  Label,
  Input,
  Switch,
  makeStyles,
  Spinner,
} from '@fluentui/react-components';
import { useProxyStore } from '../stores/proxyStore';

const useStyles = makeStyles({
  form: {
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
    maxWidth: '500px',
    marginTop: '16px',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  },
});

export default function SettingsPage() {
  const styles = useStyles();
  const { config, loading, error, fetchConfig, updateConfig } = useProxyStore();

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  if (!config) return <Spinner />;

  return (
    <div>
      <Title1>Proxy Settings</Title1>
      {loading && <Spinner />}
      {error && <div style={{ color: 'red' }}>{error}</div>}
      <div className={styles.form}>
        <div className={styles.field}>
          <Label>Port</Label>
          <Input
            type="number"
            value={String(config.port)}
            onChange={(e) => updateConfig({ ...config, port: parseInt(e.target.value) || 11888 })}
          />
        </div>
        <div className={styles.field}>
          <Switch
            label="OpenAI Protocol Enabled"
            checked={config.openai_enabled}
            onChange={(e) => updateConfig({ ...config, openai_enabled: e.target.checked })}
          />
        </div>
        <div className={styles.field}>
          <Switch
            label="Anthropic Protocol Enabled"
            checked={config.anthropic_enabled}
            onChange={(e) => updateConfig({ ...config, anthropic_enabled: e.target.checked })}
          />
        </div>
        <div className={styles.field}>
          <Switch
            label="Auto Start"
            checked={config.auto_start}
            onChange={(e) => updateConfig({ ...config, auto_start: e.target.checked })}
          />
        </div>
        <div className={styles.field}>
          <Switch
            label="Log Requests"
            checked={config.log_requests}
            onChange={(e) => updateConfig({ ...config, log_requests: e.target.checked })}
          />
        </div>
        <div className={styles.field}>
          <Label>Log Retention Days</Label>
          <Input
            type="number"
            value={String(config.log_retention_days)}
            onChange={(e) => updateConfig({ ...config, log_retention_days: parseInt(e.target.value) || 30 })}
          />
        </div>
        <div className={styles.field}>
          <Label>Max Retries</Label>
          <Input
            type="number"
            value={String(config.max_retries)}
            onChange={(e) => updateConfig({ ...config, max_retries: parseInt(e.target.value) || 3 })}
          />
        </div>
        <div className={styles.field}>
          <Label>Timeout (seconds)</Label>
          <Input
            type="number"
            value={String(config.timeout_secs)}
            onChange={(e) => updateConfig({ ...config, timeout_secs: parseInt(e.target.value) || 120 })}
          />
        </div>
      </div>
    </div>
  );
}
