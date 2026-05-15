import { useEffect } from 'react';
import { Title1, Card, CardHeader, makeStyles, tokens } from '@fluentui/react-components';
import { useProxyStore } from '../stores/proxyStore';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';

const useStyles = makeStyles({
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: '16px',
    marginTop: '16px',
  },
  card: {
    minHeight: '120px',
  },
  statusRunning: {
    color: tokens.colorPaletteGreenForeground1,
  },
  statusStopped: {
    color: tokens.colorPaletteRedForeground1,
  },
});

export default function Dashboard() {
  const styles = useStyles();
  const { config, running, getStatus, startProxy, stopProxy } = useProxyStore();
  const { providers, fetchProviders } = useProviderStore();
  const { models, fetchModels } = useModelStore();

  useEffect(() => {
    getStatus();
    fetchProviders();
    fetchModels();
  }, [getStatus, fetchProviders, fetchModels]);

  return (
    <div>
      <Title1>Dashboard</Title1>
      <div className={styles.grid}>
        <Card className={styles.card}>
          <CardHeader
            header={<b>Proxy Status</b>}
            description={
              <div>
                <span className={running ? styles.statusRunning : styles.statusStopped}>
                  {running ? '● Running' : '● Stopped'}
                </span>
                {config && <div>Port: {config.port}</div>}
                <button onClick={running ? stopProxy : startProxy}>
                  {running ? 'Stop' : 'Start'}
                </button>
              </div>
            }
          />
        </Card>
        <Card className={styles.card}>
          <CardHeader
            header={<b>Providers</b>}
            description={<div>Total: {providers.length}</div>}
          />
        </Card>
        <Card className={styles.card}>
          <CardHeader
            header={<b>Models</b>}
            description={<div>Total: {models.length}</div>}
          />
        </Card>
      </div>
    </div>
  );
}
